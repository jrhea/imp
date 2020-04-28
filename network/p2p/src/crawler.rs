use clap::{App, AppSettings, Arg, ArgMatches};
use futures_01::prelude::*;
use futures_01::stream::Stream;
use libp2p::core::{
    muxing::StreamMuxerBox, nodes::Substream, transport::dummy::DummyTransport, PeerId,
};
use libp2p::discv5::{enr, Discv5, Discv5Config, Discv5ConfigBuilder};
use libp2p::identity;
use eth2::utils::{get_fork_id_from_enr};
use slog::{debug, info, o, trace, warn};
use std::collections::HashMap;
use std::convert::TryInto;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

pub type Libp2pStream = DummyTransport<(PeerId, StreamMuxerBox)>;
pub type Discv5Stream = Discv5<Substream<StreamMuxerBox>>;
pub type Swarm = libp2p::Swarm<Libp2pStream, Discv5Stream>;
pub type Crawler = (
    Option<Swarm>,
    Option<tokio_01::sync::oneshot::Sender<()>>,
    Option<tokio_01::sync::oneshot::Receiver<()>>,
);

pub fn init(arg_matches: &ArgMatches<'_>, log: slog::Logger) -> Crawler {
    // get mothra subcommand args matches
    let mothra_arg_matches = &arg_matches.subcommand_matches("crawler").unwrap();

    let listen_address = mothra_arg_matches
        .value_of("listen-address")
        .expect("required parameter")
        .parse::<IpAddr>()
        .expect("Invalid listening address");

    let listen_port = mothra_arg_matches
        .value_of("port")
        .expect("required parameter")
        .parse::<u16>()
        .expect("Invalid listening port");

    let bootstrap_enr_str: String = if mothra_arg_matches.is_present("boot-nodes") {
        let boot_enr_list = mothra_arg_matches.value_of("boot-nodes").unwrap();
        boot_enr_list
            .split(',')
            .map(|x| x.into())
            .collect::<Vec<String>>()[0]
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

    // default configuration
    
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

    // if we know of another peer's ENR, add it known peers
    let _ = match bootstrap_enr_str
        .parse::<enr::Enr<enr::CombinedKey>>()
        .expect("Invalid base64 encoded ENR")
    {
        enr => swarm.add_enr(enr),
    };

    let (tx, rx) = tokio_01::sync::oneshot::channel::<()>();
    (Some(swarm), Some(tx), Some(rx))
}

pub async fn find_nodes(
    mut swarm: Swarm,
    mut shutdown_rx: tokio_01::sync::oneshot::Receiver<()>,
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

    let target_random_node_id = enr::NodeId::random();
    swarm.find_node(target_random_node_id);
    // construct a time interval to search for new peers.
    let mut query_interval = tokio_01::timer::Interval::new_interval(Duration::from_secs(3));
    info!(
        log,
        "{0: <6}{1: <14}{2: <55}{3: <12}{4: <69}", "index", "node_id", "peer_id", "fork_digest", "multiaddrs",
    );

    let mut peers: HashMap<String, (String, String)> = Default::default();

    tokio_01::run(futures_01::future::poll_fn(move || -> Result<_, ()> {
        loop {
            if let Ok(Async::Ready(_)) | Err(_) = shutdown_rx.poll() {
                warn!(log, "crawler: shutdown message received.");
                return Ok(Async::Ready(()));
            }
            while let Ok(Async::Ready(_)) = query_interval.poll() {
                // pick a random node target
                let target_random_node_id = enr::NodeId::random();
                //println!("Connected Peers: {}", swarm.connected_peers());

                for enr in swarm.enr_entries() {
                    let node_id = enr.node_id().clone().to_string();

                    if !peers.contains_key(&node_id) {
                        let peer_id = enr.peer_id().clone().to_string();
                        let multiaddr: String = enr
                            .multiaddr()
                            .iter()
                            .map(|m| m.to_string() + "    ")
                            .collect();
                        peers.insert(node_id.clone(), (peer_id.clone(), multiaddr.clone()));
                        let fork_id = get_fork_id_from_enr(enr).unwrap();
                        info!(
                            log,
                            "{0: <6}{1: <14}{2: <55}{3: <12}{4: <69}",
                            peers.len(),
                            node_id,
                            peer_id,
                            hex::encode(&fork_id.fork_digest),
                            multiaddr,
                        );
                    }
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