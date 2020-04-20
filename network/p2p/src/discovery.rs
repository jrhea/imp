use clap::ArgMatches;
use futures::prelude::*;
use libp2p::discv5::{enr, Discv5, Discv5Config, Discv5Event};
use libp2p::identity;
use std::collections::HashMap;
use std::convert::TryInto;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

pub fn discover_peers(arg_matches: &ArgMatches<'_>) {
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
    println!("Local Node Id: {}", local_enr.node_id());
    println!("Local Peer Id: {}", local_enr.peer_id());

    // unused transport for building a swarm
    let transport = libp2p::build_development_transport(keypair.clone());

    // default configuration
    let config = Discv5Config::default();

    // the address to listen on
    let socket_addr = SocketAddr::new(listen_address, listen_port);

    // construct the discv5 swarm, initializing an unused transport layer
    let discv5 = Discv5::new(local_enr, keypair.clone(), config, socket_addr).unwrap();
    let mut swarm = libp2p::Swarm::new(transport, discv5, keypair.public().into_peer_id());

    // if we know of another peer's ENR, add it known peers
    match bootstrap_enr_str
        .parse::<enr::Enr<enr::CombinedKey>>()
        .expect("Invalid base64 encoded ENR")
    {
        enr => {
            println!(
                "Bootstrap ENR. ip: {:?}, udp_port {:?}, tcp_port: {:?}",
                enr.ip(),
                enr.udp(),
                enr.tcp()
            );
            let _ = swarm.add_enr(enr);
        }
    };

    let target_random_node_id = enr::NodeId::random();
    swarm.find_node(target_random_node_id);

    // construct a 30 second interval to search for new peers.
    let mut query_interval = tokio_01::timer::Interval::new_interval(Duration::from_secs(3));
    println!(
        "{0: <6}{1: <14}{2: <55}{3: <69}{4: }",
        "index", "peer_id", "node_id", "multiaddrs", "enr"
    );

    let mut peers: HashMap<String, (String, String)> = Default::default();
    // Kick it off!
    tokio_01::run(futures::future::poll_fn(move || -> Result<_, ()> {
        loop {
            // start a query if it's time to do so
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
                        //let eth2 = enr.get("eth2");
                        println!(
                            "{0: <6}{1: <14}{2: <55}{3: <69}{4: }",
                            peers.len(),
                            node_id,
                            peer_id,
                            multiaddr,
                            enr.to_base64()
                        );
                    }
                }
                //println!("Searching for peers...");
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
