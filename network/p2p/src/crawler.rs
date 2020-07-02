use csv;
#[macro_use]
use serde_derive::{Serialize};
use chrono::Local;
use clap::{App, AppSettings, Arg, ArgMatches};
#[cfg(not(feature = "local"))]
use discv5::{
    enr::{CombinedKey, Enr, EnrBuilder, EnrError, NodeId},
    Discv5, Discv5Config, Discv5ConfigBuilder, Discv5Event,
};
#[cfg(feature = "local")]
use discv5_local::{
    enr::{CombinedKey, Enr, EnrBuilder, EnrError, NodeId},
    Discv5, Discv5Config, Discv5ConfigBuilder, Discv5Event,
};

use eth2::ssz::{Decode, Encode};
use eth2::utils::{
    get_attnets_from_enr, get_bitfield_from_enr, get_fork_id, get_fork_id_from_enr,
    get_fork_id_from_string,EnrExt
};
use futures::future::Future;
use futures::prelude::*;
use rand::Rng;
use slog::{debug, info, o, trace, warn};
use std::any::type_name;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::watch;
use types::events::Events;

#[derive(Serialize, Default, Clone)]
struct EnrEntry {
    node_id: String,
    peer_id: String,
    ip4: String,
    tcp4: String,
    udp4: String,
    ip6: String,
    tcp6: String,
    udp6: String,
    next_fork_version: String,
    next_fork_epoch: String,
    fork_digest: String,
    seq_no: String,
    subnet_ids: String,
    enr: String,
}

impl EnrEntry {
    pub fn new(enr: &Enr<CombinedKey>) -> EnrEntry {
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
        let peer_id = enr.peer_id().to_string();
        let seq_no = enr.seq().clone().to_string();
        let fork_id = get_fork_id_from_enr(enr);
        let (next_fork_version, next_fork_epoch, fork_digest) = match fork_id {
            Some(x) => (
                hex::encode(x.next_fork_version),
                format!("{}", x.next_fork_epoch),
                hex::encode(&x.fork_digest),
            ),
            _ => ("".to_string(), "".to_string(), "".to_string()),
        };
        let subnet_ids = format!("{:?}", get_attnets_from_enr(enr));
        EnrEntry {
            node_id: node_id.clone(),
            peer_id: peer_id.clone(),
            ip4: ip4.clone(),
            tcp4: tcp4.clone(),
            udp4: udp4.clone(),
            ip6: ip6.clone(),
            tcp6: tcp6.clone(),
            udp6: udp6.clone(),
            next_fork_version: next_fork_version.clone(),
            next_fork_epoch: next_fork_epoch.clone(),
            fork_digest: fork_digest.clone(),
            seq_no: seq_no.clone(),
            subnet_ids: subnet_ids.clone(),
            enr: enr.to_base64(),
        }
    }
}

#[derive(Serialize, Default, Clone)]
struct EnrRecord {
    index: u32,
    timestamp: String,
    #[serde(skip_serializing)]
    enr: EnrEntry,
}

impl EnrRecord {
    pub fn new(index: u32, timestamp: String, enr_entry: EnrEntry) -> EnrRecord {
        EnrRecord {
            index,
            timestamp,
            enr: enr_entry,
        }
    }
    pub fn get_enr(&self) -> Option<Enr<CombinedKey>> {
        match self.enr.enr.parse::<Enr<CombinedKey>>() {
            Ok(x) => Some(x),
            _ => None,
        }
    }
}

#[derive(Serialize, Default)]
struct FindNodeRecord {
    index: u32,
    timestamp: String,
    target_node_id: String,
    closer_peers: Vec<String>,
}
#[derive(Serialize, Default)]
struct DiscoveredRecord {
    index: u32,
    timestamp: String,
    enr: EnrEntry,
}
#[derive(Serialize, Default)]
struct NodeInsertedRecord {
    index: u32,
    timestamp: String,
    node_id: String,
    replaced: String,
}
#[derive(Serialize, Default)]
struct EnrAddedRecord {
    index: u32,
    timestamp: String,
    enr: EnrEntry,
    replaced: EnrEntry,
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
    pub fn new(arg_matches: &ArgMatches<'_>, log: slog::Logger) -> Self {
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

        for enr in &boot_enr_list {
            match get_fork_id_from_string(enr.to_string()) {
                Some(x) => {
                    if hex::encode(&x.fork_digest) == fork_digest {
                        info!(
                            log,
                            "fork_digest:{:?},next_fork_version:{:?},next_fork_epoch:{:?}",
                            hex::encode(&x.fork_digest),
                            hex::encode(x.next_fork_version),
                            u64::max_value()
                        );
                        break;
                    }
                }
                _ => (),
            }
        }

        fn filter(enr: &Enr<CombinedKey>) -> bool {
            let fork_id = get_fork_id(
                hex::decode("f6775d07").unwrap(),
                hex::decode("00000113").unwrap(),
                u64::max_value(),
            );
            enr.get("eth2") == Some(&fork_id.as_ssz_bytes().clone())
        };

        let config = Discv5ConfigBuilder::new()
            .request_timeout(Duration::from_secs(4))
            .request_retries(1) //default 1
            .enr_peer_update_min(5) // prevents NAT's should be raised for mainnet   //default 10
            .query_parallelism(10) //default 3
            .query_peer_timeout(Duration::from_secs(2)) //default 2
            .query_timeout(Duration::from_secs(10)) //default 60
            .session_timeout(Duration::from_secs(86400)) //default 86400
            //.table_filter(filter)
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

    pub async fn find_nodes(self, mut shutdown_rx: watch::Receiver<Events>, log: slog::Logger) {
        // construct the discv5 swarm, initializing an unused transport layer
        let mut discv5 =
            Discv5::new(self.local_enr, self.enr_key, self.config).unwrap();
        // start the discv5 service
        discv5.start(self.socket_addr);
        // if we know of another peer's ENR, add it known peers
        for enr_str in self.boot_enr_list {
            let _ = match enr_str.parse::<Enr<CombinedKey>>() {
                Ok(enr) => {
                    trace!(log, "Added {} to list of bootstrap enrs", enr_str);
                    info!(
                        log,
                        "Bootstrap ENR. ip: {:?}, udp_port {:?}, tcp_port: {:?}",
                        enr.ip(),
                        enr.udp(),
                        enr.tcp()
                    );
                    discv5.add_enr(enr)
                }
                Err(_) => {
                    trace!(log, "Failed to add {} to list of bootstrap enrs", enr_str);
                    Ok(())
                }
            };
        }

        let output_file = match discv5.local_enr().udp() {
            Some(x) => format!("crawler{}.csv", x),
            _ => format!("crawler.csv"),
        };
        let mut target_enr = "".to_string();
        let target_fork_digest = self.fork_digest;
        // construct a time interval to search for new peers.
        let mut query_interval = tokio::time::interval(Duration::from_secs(10));
        let mut output_interval = tokio::time::interval(Duration::from_secs(30));
        let mut enr_records: HashMap<String, EnrRecord> = Default::default();

        let mut enr_added_count: u64 = 0;
        let mut node_inserted_count: u64 = 0;
        let mut event_stream = discv5.event_stream().await.unwrap();
        loop {
            tokio::select! {
                x = shutdown_rx.recv() => {
                    if let Some(Events::ShutdownMessage) = x {
                        warn!(
                            log,
                            "{:?}: shutdown message received.",
                            type_name::<Crawler>()
                        );
                        match self.output_mode.as_str() {
                            "snapshot" => {
                                info!(log,"Output is enabled.  Saving data to file");
                                Crawler::write_file(&enr_records, self.datadir.join(&output_file));
                                break;
                            },
                            _ => {
                                info!(log,"Output is disabled.  Not saving to file.");
                                break
                            }
                        };
                    }
                },
                _ = output_interval.next() => {
                    match self.output_mode.as_str() {
                        "snapshot" => {
                            info!(log,"Output is enabled.  Saving data to file");
                            Crawler::write_file(&enr_records, self.datadir.join(&output_file));
                        },
                        _ => ()
                    }
                },
                _ = query_interval.next() => {
                    let mut index = 1;
                    let timestamp = format!("{}", Local::now().format("%Y-%m-%d][%H:%M:%S"));
                    // pick a random node target
                    let target_random_node_id = NodeId::random();

                    //let mut rng = rand::thread_rng();
                    //let rnum: f64 = rng.gen();
                    let rnum = 0.30;
                    let enrs = if enr_added_count % 2 == 0 {
                        let node_ids_discovered: Vec<String> = enr_records.keys().cloned().collect();
                        let x = target_fork_digest.clone();
                        // predicate for finding nodes with a matching fork_digest
                        let eth2_fork_predicate =
                            move |enr: &Enr<CombinedKey>| {
                                let enr_node_id = hex::encode(enr.node_id().raw());
                                let enr_fork_digest = match get_fork_id_from_enr(&enr) {
                                    Some(fork_id) => hex::encode(&fork_id.fork_digest),
                                    _ => "".to_string(),
                                };
                                hex::encode(&x) == enr_fork_digest
                                && !node_ids_discovered.contains(&enr_node_id)
                            };
                        let predicate = move |enr: &Enr<CombinedKey>| {
                            eth2_fork_predicate(enr)
                        };
                        info!(log,"calling find_node_predicate()");
                        discv5.find_node_predicate(
                            target_random_node_id,
                            Box::new(predicate),
                            256,
                        ).await
                    } else {
                        info!(log,"calling find_node()");
                        discv5.find_node(target_random_node_id).await
                    };


                    for enr in discv5.table_entries_enr() {
                        let enr_entry = EnrEntry::new(&enr);
                        if target_enr == "".to_string()
                        && (enr_entry.fork_digest == target_fork_digest || target_fork_digest.is_empty())
                        {
                            target_enr = enr.to_base64();
                        }
                        let enr_record = enr_records.entry(enr_entry.node_id.clone()).or_default();
                        *enr_record = EnrRecord::new(index, timestamp.clone(), enr_entry);
                        index += 1;
                        enr_added_count += 1;
                    }
                    info!(log, "Connected Peers: {}", discv5.connected_peers());
                    info!(log, "Enr Entries: {:?}", enr_records.len());
                    if target_enr != "".to_string() {
                        let fork_id = get_fork_id_from_enr(
                            &target_enr.parse::<Enr<CombinedKey>>().unwrap(),
                        );
                        match fork_id {
                            Some(x) => {
                                //info!(log,"rnum:{}",rnum);
                                if target_fork_digest.is_empty() || target_fork_digest == hex::encode(&x.fork_digest)
                                {
                                    let num_on_fork = enr_records.values().filter(|enr_record| {
                                        match enr_record.get_enr() {
                                            Some(y) => {
                                                let enr_record_fork_digest = match get_fork_id_from_enr(&y) {
                                                    Some(fork_id) => hex::encode(&fork_id.fork_digest),
                                                    _ => "".to_string(),
                                                };
                                                let fork_id_match = hex::encode(&x.fork_digest) == enr_record_fork_digest;
                                                //info!(log,"fork_digest:{} {:?}",enr_record_fork_digest,fork_id_match);
                                                fork_id_match
                                            },
                                            _ => false
                                        }
                                    }).count();
                                    info!(log, "Enr Entries on correct fork_digest: {:?}", num_on_fork);
                                } 
                            }
                            _ => ()
                        }
                    } 
                }
            }
        }
    }

    fn write_file(records: &HashMap<String, EnrRecord>, path: PathBuf) {
        let file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .append(false)
            .open(path)
            .unwrap();
        let mut wtr = csv::Writer::from_writer(file);

        for enr_record in records.values() {
            let _ = wtr.serialize((&enr_record, &enr_record.enr));
            let _ = wtr.flush();
        }
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
            .possible_values(&["snapshot","none"])
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