use clap::ArgMatches;
use futures_01::prelude::*;
use futures_01::stream::Stream;
use libp2p::core::{
    muxing::StreamMuxerBox, nodes::Substream, transport::dummy::DummyTransport, PeerId,
};
use libp2p::discv5::{enr, Discv5, Discv5Config};
use libp2p::identity;
use slog::{debug, info, o, trace, warn};
use std::collections::HashMap;
use std::convert::TryInto;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

pub type Libp2pStream = DummyTransport<(PeerId, StreamMuxerBox)>;
pub type Discv5Stream = Discv5<Substream<StreamMuxerBox>>;
pub type Swarm = libp2p::Swarm<Libp2pStream, Discv5Stream>;
pub type Discovery = (
    Option<Swarm>,
    Option<tokio_01::sync::oneshot::Sender<()>>,
    Option<tokio_01::sync::oneshot::Receiver<()>>,
);

pub fn init(arg_matches: &ArgMatches<'_>, log: slog::Logger) -> Discovery {
    // get mothra subcommand args matches
    let mothra_arg_matches = &arg_matches.subcommand_matches("mothra").unwrap();

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
    let config = Discv5Config::default();

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
        "{0: <6}{1: <14}{2: <55}{3: <69}", "index", "peer_id", "node_id", "multiaddrs",
    );

    let mut peers: HashMap<String, (String, String)> = Default::default();

    tokio_01::run(futures_01::future::poll_fn(move || -> Result<_, ()> {
        loop {
            if let Ok(Async::Ready(_)) | Err(_) = shutdown_rx.poll() {
                warn!(log, "discovery: shutdown message received.");
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

                        info!(
                            log,
                            "{0: <6}{1: <14}{2: <55}{3: <69}",
                            peers.len(),
                            node_id,
                            peer_id,
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
