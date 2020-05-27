use csv;
#[macro_use]
use serde_derive::{Serialize};
use chrono::Local;
use clap::{App, AppSettings, Arg, ArgMatches};
use eth2::ssz::{Decode, Encode};
use eth2::utils::{get_attnets_from_enr, get_bitfield_from_enr, get_fork_id_from_enr};
use futures::prelude::*;
use futures::future::Future;
use tokio_02::sync::watch;
use discv5::{enr::{CombinedKey, Enr, EnrBuilder, EnrError, NodeId}, Discv5, Discv5Event, Discv5Config, Discv5ConfigBuilder};
use slog::{debug, info, o, trace, warn};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use types::events::Events;
use std::any::type_name;

#[derive(Serialize, Default)]
struct Record {
    index: u32,
    timestamp: String,
    node_id: String,
    ip4: String,
    tcp4: String,
    udp4: String,
    ip6: String,
    tcp6: String,
    udp6: String,
    fork_digest: String,
    seq_no: String,
    subnet_ids: String,
    enr: String,
}

pub struct Crawler {
    local_enr: Enr<CombinedKey>,
    enr_key: CombinedKey,
    socket_addr: SocketAddr,
    boot_enr_list: Vec<String>,
    config: Discv5Config,
    output_mode: String,
    fork_digest: String,
    datadir: PathBuf,
}

impl Crawler {
    pub fn new(
        arg_matches: &ArgMatches<'_>,
        log: slog::Logger,
    ) -> Self {
        // get mothra subcommand args matches
        let crawler_arg_matches = &arg_matches.subcommand_matches("crawler").unwrap();

        let output_mode = crawler_arg_matches
            .value_of("output-mode")
            .expect("required parameter");

        let datadir = crawler_arg_matches
            .value_of("datadir")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".imp")
                    .join("output")
            });

        let listen_address = crawler_arg_matches
            .value_of("listen-address")
            .expect("required parameter")
            .parse::<IpAddr>()
            .expect("Invalid listening address");

        let listen_port = crawler_arg_matches
            .value_of("port")
            .expect("required parameter")
            .parse::<u16>()
            .expect("Invalid listening port");

        let fork_digest = crawler_arg_matches
            .value_of("fork-digest")
            .expect("required parameter");

        let boot_enr_list = if crawler_arg_matches.is_present("boot-nodes") {
            crawler_arg_matches
                .value_of("boot-nodes")
                .unwrap()
                .split(',')
                .map(|x| x.into())
                .collect::<Vec<String>>()
                .clone()
        } else {
            Default::default()
        };
        info!(log, "Found {} bootstrap enrs", boot_enr_list.len());

        // build the local ENR
        let enr_key = CombinedKey::generate_secp256k1();
        let local_enr = {
            EnrBuilder::new("v4")
                .ip(listen_address)
                .udp(listen_port)
                .build(&enr_key)
                .unwrap()
        };
        info!(log, "Local Node Id: {}", local_enr.node_id());
        //info!(log, "Local Peer Id: {}", local_enr.peer_id());

        let config = Discv5ConfigBuilder::new()
            .request_timeout(Duration::from_secs(4))
            .request_retries(1) //default 1
            .enr_update(true) // update IP based on PONG responses
            .enr_peer_update_min(5) // prevents NAT's should be raised for mainnet   //default 10
            .query_parallelism(10) //default 3
            .query_peer_timeout(Duration::from_secs(2)) //default 2
            .query_timeout(Duration::from_secs(60)) //default 60
            .session_timeout(Duration::from_secs(86400)) //default 86400
            .session_establish_timeout(Duration::from_secs(15)) //default 15
            .ip_limit(false) // limits /24 IP's in buckets. Enable for mainnet
            .ping_interval(Duration::from_secs(300))
            .build();

        // the address to listen on
        let socket_addr = SocketAddr::new(listen_address, listen_port);

        Crawler {
            local_enr,
            enr_key,
            socket_addr,
            boot_enr_list,
            config,
            output_mode: output_mode.to_string(),
            fork_digest: fork_digest.to_string(),
            datadir,
        }
    }

    pub async fn find_nodes(mut self, mut shutdown_rx: watch::Receiver<Events>, log: slog::Logger) {

        // construct the discv5 swarm, initializing an unused transport layer
        let mut discv5 = Discv5::new(self.local_enr, self.enr_key, self.config, self.socket_addr).unwrap();
        // if we know of another peer's ENR, add it known peers
        for enr_str in self.boot_enr_list {
            let _ = match enr_str.parse::<Enr<CombinedKey>>() {
                Ok(enr) => {
                    trace!(log, "Added {} to list of bootstrap enrs", enr_str);
                    discv5.add_enr(enr)
                }
                Err(_) => {
                    trace!(log, "Failed to add {} to list of bootstrap enrs", enr_str);
                    Ok(())
                }
            };
        }
        if let Some(enr) = discv5.enr_entries().next() {
            info!(
                log,
                "Bootstrap ENR. ip: {:?}, udp_port {:?}, tcp_port: {:?}",
                enr.ip(),
                enr.udp(),
                enr.tcp()
            );
        }

        let output_file = match discv5.local_enr().udp() {
            Some(x) => format!("crawler{}.csv", x),
            _ => format!("crawler.csv"),
        };
        let mut target_enr = "".to_string();
        let target_random_node_id = NodeId::random();
        discv5.find_node(target_random_node_id);
        // construct a time interval to search for new peers.
        let mut query_interval = tokio_02::time::interval(Duration::from_secs(5));
        let mut peers: HashMap<String, Record> = Default::default();

        loop {
            // if let Some(Events::ShutdownMessage) = shutdown_rx.recv().await {
            //     warn!(
            //         log,
            //         "{:?}: shutdown message received.",
            //         type_name::<Crawler>()
            //     );
            //     break;
            // }
            tokio_02::select! {
                _ = query_interval.next() => {
                    // pick a random node target
                    let target_random_node_id = NodeId::random();
                    info!(log, "Connected Peers: {}", discv5.connected_peers());
                    info!(log, "Searching for peers...");
                    // execute a FINDNODE query
                    discv5.find_node(target_random_node_id);
                }
                Some(event) = discv5.next() => {
                        if let Discv5Event::FindNodeResult { closer_peers, .. } = event {
                            if !closer_peers.is_empty() {
                                info!(log, "Query Completed. Nodes found:");
                                for n in closer_peers {
                                    info!(log, "Node: {}", n);
                                }
                            } else {
                                info!(log, "Query Completed. No peers found.")
                            }
                        }
                    }
            }
        }

/*        tokio_01::run(futures_01::future::poll_fn(move || -> Result<_, ()> {
            loop {
                if let Ok(Async::Ready(_)) | Err(_) = shutdown_rx.poll() {
                    warn!(log, "crawler: shutdown message received.");
                    return Ok(Async::Ready(()));
                }
                while let Ok(Async::Ready(_)) = query_interval.poll() {
                    let file = match self.output_mode.as_str() {
                        "timehistory" => OpenOptions::new()
                            .write(true)
                            .create(true)
                            .append(true)
                            .open(self.datadir.join(&output_file))
                            .unwrap(),
                        _ => OpenOptions::new()
                            .truncate(true)
                            .write(true)
                            .create(true)
                            .append(false)
                            .open(self.datadir.join(&output_file))
                            .unwrap(),
                    };
                    let mut wtr = csv::Writer::from_writer(file);
                    let mut index = 1;
                    let timestamp = format!("{}", Local::now().format("%Y-%m-%d][%H:%M:%S"));
                    // pick a random node target
                    let target_random_node_id = NodeId::random();
                    //println!("Connected Peers: {}", swarm.connected_peers());

                    for enr in self.discv5.enr_entries() {
                        let ip4: String = match enr.ip() {
                            Some(x) => x.to_string(),
                            _ => "".to_string(),
                        };
                        let tcp4: String = match enr.tcp() {
                            Some(x) => x.to_string(),
                            _ => "".to_string(),
                        };
                        let udp4: String = match enr.udp() {
                            Some(x) => x.to_string(),
                            _ => "".to_string(),
                        };
                        let ip6: String = match enr.ip6() {
                            Some(x) => x.to_string(),
                            _ => "".to_string(),
                        };
                        let tcp6: String = match enr.tcp6() {
                            Some(x) => x.to_string(),
                            _ => "".to_string(),
                        };
                        let udp6: String = match enr.udp6() {
                            Some(x) => x.to_string(),
                            _ => "".to_string(),
                        };
                        let node_id = hex::encode(enr.node_id().clone().raw());
                         let seq_no = enr.seq().clone().to_string();
                        let fork_id = get_fork_id_from_enr(enr);
                        let fork_digest = match fork_id {
                            Some(x) => hex::encode(&x.fork_digest),
                            _ => "".to_string(),
                        };
                        if target_enr == "".to_string()
                            && (fork_digest == self.fork_digest || self.fork_digest.is_empty())
                        {
                            target_enr = enr.to_base64();
                        }
                        let subnet_ids = format!("{:?}", get_attnets_from_enr(enr));
                        let record = peers.entry(node_id.clone()).or_default();
                        *record = Record {
                            index,
                            timestamp: timestamp.clone(),
                            node_id: node_id.clone(),
                            ip4: ip4.clone(),
                            tcp4: tcp4.clone(),
                            udp4: udp4.clone(),
                            ip6: ip6.clone(),
                            tcp6: tcp6.clone(),
                            udp6: udp6.clone(),
                            fork_digest: fork_digest.clone(),
                            seq_no: seq_no.clone(),
                            subnet_ids: subnet_ids.clone(),
                            enr: enr.to_base64(),
                        };
                        let _ = wtr.serialize(record);
                        let _ = wtr.flush();
                        index += 1;
                    }
                    if target_enr != "".to_string() {
                        let fork_id = get_fork_id_from_enr(
                            &target_enr.parse::<Enr<CombinedKey>>().unwrap(),
                        );
                        match fork_id {
                            Some(x) => {
                                if self.fork_digest.is_empty()
                                    || self.fork_digest == hex::encode(&x.fork_digest)
                                {
                                    let enr_fork_id = x.as_ssz_bytes();
                                    // predicate for finding nodes with a matching fork
                                    let eth2_fork_predicate =
                                        move |enr: &Enr<CombinedKey>| {
                                            enr.get("eth2") == Some(&enr_fork_id.clone())
                                        };
                                    let predicate = move |enr: &Enr<CombinedKey>| {
                                        eth2_fork_predicate(enr)
                                    };
                                    self.discv5.find_enr_predicate(
                                        target_random_node_id,
                                        predicate,
                                        32,
                                    )
                                } else {
                                    self.discv5.find_node(target_random_node_id)
                                }
                            }
                            _ => (),
                        };
                    } else {
                        self.discv5.find_node(target_random_node_id);
                    }
                }

                match self.discv5.poll().expect("Error while polling swarm") {
                    Async::Ready(Some(event)) => match event {
                        _ => (),
                    },
                    Async::Ready(None) | Async::NotReady => break,
                }
            }
            Ok(Async::NotReady)
        }));*/
    }
}

pub fn cli_app<'a, 'b>() -> App<'a, 'b> {
    App::new("crawler")
    .version(clap::crate_version!())
    .about("ETH2 network crawler.")
    .arg(
        Arg::with_name("output-mode")
            .long("output-mode")
            .allow_hyphen_values(true)
            .value_name("OUTPUT-MODE")
            .help("Controls how data is collected and output.")
            .takes_value(true)
            .possible_values(&["snapshot","timehistory"])
            .default_value("snapshot"),
    )
    .arg(
        Arg::with_name("datadir")
            .long("datadir")
            .value_name("DIR")
            .help("The location of the data directory to use.")
            .takes_value(true)
    )
    .arg(
        Arg::with_name("listen-address")
            .long("listen-address")
            .value_name("ADDRESS")
            .help("The address the client will listen for UDP and TCP connections.")
            .default_value("127.0.0.1")
            .takes_value(true),
    )
    .arg(
        Arg::with_name("port")
            .long("port")
            .value_name("PORT")
            .help("The TCP/UDP port to listen on.")
            .default_value("9000")
            .takes_value(true),
    )
    .arg(
        Arg::with_name("fork-digest")
            .long("fork-digest")
            .allow_hyphen_values(true)
            .value_name("FORK-DIGEST")
            .help("Fork digest of the network to crawl.")
            .possible_values(&["9925efd6","f071c66c",""])
            .default_value("")
            .takes_value(true),
    )
    .arg(
        Arg::with_name("boot-nodes")
            .long("boot-nodes")
            .allow_hyphen_values(true)
            .value_name("ENR-LIST")
            .help("One or more comma-delimited base64-encoded ENR's to bootstrap the p2p network.")
            .takes_value(true),
    )
}
