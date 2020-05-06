use csv;
#[macro_use]
use serde_derive::{Serialize};
use chrono::Local;
use clap::{App, AppSettings, Arg, ArgMatches};
use eth2::utils::{get_attnets_from_enr, get_bitfield_from_enr, get_fork_id_from_enr};
use futures_01::prelude::*;
use futures_01::stream::Stream;
use libp2p::core::{
    muxing::StreamMuxerBox, nodes::Substream, transport::dummy::DummyTransport, PeerId,
};
use libp2p::discv5::{enr, Discv5, Discv5Config, Discv5ConfigBuilder};
use libp2p::identity;
use slog::{debug, info, o, trace, warn};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;

pub type Libp2pStream = DummyTransport<(PeerId, StreamMuxerBox)>;
pub type Discv5Stream = Discv5<Substream<StreamMuxerBox>>;
pub type Swarm = libp2p::Swarm<Libp2pStream, Discv5Stream>;
pub type Crawler = (
    Option<Swarm>,
    Option<tokio_01::sync::oneshot::Sender<()>>,
    Option<tokio_01::sync::oneshot::Receiver<()>>,
    Option<PathBuf>,
);

//"index", "node_id", "peer_id", "ip4", "tcp4", "udp4", "ip6", "tcp6", "udp6", "enr_fork_digest", "enr_seq", "subnet_ids",

#[derive(Serialize, Default)]
struct Record {
    index: u32,
    timestamp: String,
    node_id: String,
    peer_id: String,
    ip4: String,
    tcp4: String,
    udp4: String,
    ip6: String,
    tcp6: String,
    udp6: String,
    fork_digest: String,
    seq_no: String,
    subnet_ids: String,
    enr: String
}

pub fn init(arg_matches: &ArgMatches<'_>, log: slog::Logger) -> Crawler {
    // get mothra subcommand args matches
    let crawler_arg_matches = &arg_matches.subcommand_matches("crawler").unwrap();

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
    // build the local ENR
    let keypair = identity::Keypair::generate_secp256k1();
    let enr_key = keypair.clone().try_into().unwrap();
    let local_enr = {
        enr::EnrBuilder::new("v4")
            .ip(listen_address)
            .udp(listen_port)
            .build(&enr_key)
            .unwrap()
    };
    info!(log, "Local Node Id: {}", local_enr.node_id());
    info!(log, "Local Peer Id: {}", local_enr.peer_id());

    // unused transport for building a swarm
    let transport: Libp2pStream = libp2p::core::transport::dummy::DummyTransport::new();

    let config = Discv5ConfigBuilder::new()
        .request_timeout(Duration::from_secs(4))
        .request_retries(2) //default 1
        .enr_update(true) // update IP based on PONG responses
        .enr_peer_update_min(5) // prevents NAT's should be raised for mainnet   //default 10
        .query_parallelism(5) //default 3
        .query_peer_timeout(Duration::from_secs(2)) //default 2
        .query_timeout(Duration::from_secs(60)) //default 60
        .session_timeout(Duration::from_secs(86400)) //default 86400
        .session_establish_timeout(Duration::from_secs(15)) //default 15
        .ip_limit(false) // limits /24 IP's in buckets. Enable for mainnet
        .ping_interval(Duration::from_secs(300))
        .build();

    // the address to listen on
    let socket_addr = SocketAddr::new(listen_address, listen_port);

    // construct the discv5 swarm, initializing an unused transport layer
    let discv5 = Discv5::new(local_enr, keypair.clone(), config, socket_addr).unwrap();
    let mut swarm: Swarm = libp2p::Swarm::new(transport, discv5, keypair.public().into_peer_id());

    info!(log,"Found {} bootstrap enrs", boot_enr_list.len());
    // if we know of another peer's ENR, add it known peers
    for enr_str in boot_enr_list {
        let _ = match enr_str
            .parse::<enr::Enr<enr::CombinedKey>>()
        {
            Ok(enr) => {
                trace!(log, "Added {} to list of bootstrap enrs", enr_str);
                swarm.add_enr(enr)
            },
            Err(_) => {
                trace!(log, "Failed to add {} to list of bootstrap enrs", enr_str);
                Ok(())
            }
        };
    }

    let (tx, rx) = tokio_01::sync::oneshot::channel::<()>();
    (Some(swarm), Some(tx), Some(rx), Some(datadir))
}

pub async fn find_nodes(
    mut swarm: Swarm,
    mut shutdown_rx: tokio_01::sync::oneshot::Receiver<()>,
    datadir: PathBuf,
    log: slog::Logger,
) {
    if let Some(enr) = swarm.enr_entries().next() {
        info!(
            log,
            "Bootstrap ENR. ip: {:?}, udp_port {:?}, tcp_port: {:?}",
            enr.ip(),
            enr.udp(),
            enr.tcp()
        );
    }

    let output_file = match swarm.local_enr().udp() {
        Some(x) => format!("crawler{}.csv", x),
        _ => format!("crawler.csv")
    };
    let target_random_node_id = enr::NodeId::random();
    swarm.find_node(target_random_node_id);
    // construct a time interval to search for new peers.
    let mut query_interval = tokio_01::timer::Interval::new_interval(Duration::from_secs(5));
    let mut peers: HashMap<String, Record> = Default::default();

    tokio_01::run(futures_01::future::poll_fn(move || -> Result<_, ()> {
        loop {
            if let Ok(Async::Ready(_)) | Err(_) = shutdown_rx.poll() {
                warn!(log, "crawler: shutdown message received.");
                return Ok(Async::Ready(()));
            }
            while let Ok(Async::Ready(_)) = query_interval.poll() {
                let file = OpenOptions::new()
                    .truncate(true)
                    .write(true)
                    .create(true)
                    .append(false)
                    .open(datadir.join(&output_file))
                    .unwrap();
                let mut wtr = csv::Writer::from_writer(file);
                let mut index = 1;
                let timestamp = format!("{}", Local::now().format("%Y-%m-%d][%H:%M:%S"));
                // pick a random node target
                let target_random_node_id = enr::NodeId::random();
                //println!("Connected Peers: {}", swarm.connected_peers());

                for enr in swarm.enr_entries() {
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
                    let peer_id = enr.peer_id().clone().to_string();
                    let seq_no = enr.seq().clone().to_string();
                    let fork_digest = match get_fork_id_from_enr(enr) {
                        Some(x) => hex::encode(&x.fork_digest),
                        _ => "".to_string(),
                    };
                    let subnet_ids = format!("{:?}", get_attnets_from_enr(enr));
                    let record = peers.entry(node_id.clone()).or_default();
                    *record = Record {
                        index,
                        timestamp: timestamp.clone(),
                        node_id: node_id.clone(),
                        peer_id: peer_id.clone(),
                        ip4: ip4.clone(),
                        tcp4: tcp4.clone(),
                        udp4: udp4.clone(),
                        ip6: ip6.clone(),
                        tcp6: tcp6.clone(),
                        udp6: udp6.clone(),
                        fork_digest: fork_digest.clone(),
                        seq_no: seq_no.clone(),
                        subnet_ids: subnet_ids.clone(),
                        enr: enr.to_base64()
                    };
                    let _ = wtr.serialize(record);
                    let _ = wtr.flush();
                    index += 1;
                }

                // execute a FINDNODE query
                swarm.find_node(target_random_node_id);
            }

            match swarm.poll().expect("Error while polling swarm") {
                Async::Ready(Some(event)) => match event {
                    _ => (),
                },
                Async::Ready(None) | Async::NotReady => break,
            }
        }
        Ok(Async::NotReady)
    }));
}

pub fn cli_app<'a, 'b>() -> App<'a, 'b> {
    App::new("crawler")
    .version(clap::crate_version!())
    .about("ETH2 network crawler.")
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
        Arg::with_name("boot-nodes")
            .long("boot-nodes")
            .allow_hyphen_values(true)
            .value_name("ENR-LIST")
            .help("One or more comma-delimited base64-encoded ENR's to bootstrap the p2p network.")
            .takes_value(true),
    )
}
