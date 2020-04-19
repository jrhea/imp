//! Demonstrates how to run a basic Discovery v5 Service.
//!
//! This example creates a libp2p discv5 service which searches for peers every 30 seconds. On
//! creation, the local ENR created for this service is displayed in base64. This can be used to
//! allow other instances to connect and join the network. The service can be stopped by pressing
//! Ctrl-C.
//!
//! To add peers to the network, create multiple instances of this service adding the ENR of a
//! participating node in the command line. The nodes should discover each other over a period of
//! time. (It is probabilistic that nodes to find each other on any given query).
//!
//! A single instance listening on a udp socket `127.0.0.1:9000` (with an ENR that has an empty IP
//! and UDP port) can be created via:
//!
//! ```
//! sh cargo run --example discv5
//! ```
//!
//! As the associated ENR has no IP/Port it is not displayed, as it cannot be used to connect to.
//!
//! An ENR IP address (to allow another nodes to dial this service), port and ENR node can also be
//! passed as command line options. Therefore, a second instance, in a new terminal, can be run on
//! port 9001 and connected to another node with a valid ENR:
//!
//! ```
//! sh cargo run --example discv5 -- 127.0.0.1 9001 <GENERATE_KEY> <BASE64_ENR>
//! ```
//!
//! where `<BASE64_ENR>` is the base64 ENR given from executing the first node with an IP and port
//! given in the CLI.
//! `<GENERATE_KEY>` is a boolean (`true` or `false`) specifying if a new key should be generated.
//! These steps can be repeated to add further nodes to the test network.
//!
//! The parameters are optional.

use futures::prelude::*;
use libp2p::discv5::{enr, Discv5, Discv5Config, Discv5Event};
use libp2p::identity;
use std::convert::TryInto;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use std::collections::HashMap;

fn main() {
    env_logger::init();

    // if there is an address specified use it
    let address = {
        if let Some(address) = std::env::args().nth(1) {
            address.parse::<Ipv4Addr>().unwrap()
        } else {
            "127.0.0.1".parse::<Ipv4Addr>().unwrap()
        }
    };

    let port = {
        if let Some(udp_port) = std::env::args().nth(2) {
            u16::from_str_radix(&udp_port, 10).unwrap()
        } else {
            9000
        }
    };

    // A fixed key for testing
    let raw_key = vec![
        183, 28, 113, 166, 126, 17, 119, 173, 78, 144, 22, 149, 225, 180, 185, 238, 23, 174, 22,
        198, 102, 141, 49, 62, 172, 47, 150, 219, 205, 163, 242, 145,
    ];
    let secret_key = identity::secp256k1::SecretKey::from_bytes(raw_key).unwrap();
    let mut keypair = identity::Keypair::Secp256k1(identity::secp256k1::Keypair::from(secret_key));

    // use a random key if specified
    if let Some(generate_key) = std::env::args().nth(3) {
        if generate_key.parse::<bool>().unwrap() {
            keypair = identity::Keypair::generate_secp256k1();
        }
    }

    // build an enr key from the libp2p key
    let enr_key = keypair.clone().try_into().unwrap();

    // construct a local ENR
    let enr = {
        let mut builder = enr::EnrBuilder::new("v4");
        // if an IP was specified, use it
        if std::env::args().nth(1).is_some() {
            builder.ip(address.into());
        }
        // if a port was specified, use it
        if std::env::args().nth(2).is_some() {
            builder.udp(port);
        }
        builder.build(&enr_key).unwrap()
    };

    // if the ENR is useful print it
    println!("Node Id: {}", enr.node_id());
    if enr.udp_socket().is_some() {
        println!("Base64 ENR: {}", enr.to_base64());
        println!("IP: {}, UDP_PORT:{}", enr.ip().unwrap(), enr.udp().unwrap());
    } else {
        println!("ENR is not printed as no IP:PORT was specified");
    }

    // unused transport for building a swarm
    let transport = libp2p::build_development_transport(keypair.clone());

    // default configuration
    let config = Discv5Config::default();

    // the address to listen on
    let socket_addr = SocketAddr::new(address.into(), port);

    // construct the discv5 swarm, initializing an unused transport layer
    let discv5 = Discv5::new(enr, keypair.clone(), config, socket_addr).unwrap();
    let mut swarm = libp2p::Swarm::new(transport, discv5, keypair.public().into_peer_id());

    // if we know of another peer's ENR, add it known peers
    if let Some(base64_enr) = std::env::args().nth(4) {
        match base64_enr.parse::<enr::Enr<enr::CombinedKey>>() {
            Ok(enr) => {
                println!(
                    "ENR Read. ip: {:?}, udp_port {:?}, tcp_port: {:?}",
                    enr.ip(),
                    enr.udp(),
                    enr.tcp()
                );
                swarm.add_enr(enr);
            }
            Err(e) => panic!("Decoding ENR failed: {}", e),
        }
    }
    let target_random_node_id = enr::NodeId::random();
    swarm.find_node(target_random_node_id);

    // construct a 30 second interval to search for new peers.
    let mut query_interval = tokio::timer::Interval::new_interval(Duration::from_secs(3));
    println!(
        "{0: <6}{1: <14}{2: <55}{3: <66}{4: }",
        "index", "peer_id", "node_id", "multiaddrs","enr"
    );

    let mut peers: HashMap<String, (String,String)> = Default::default();
    // Kick it off!
    tokio::run(futures::future::poll_fn(move || -> Result<_, ()> {
        
        loop {
            // start a query if it's time to do so
            while let Ok(Async::Ready(_)) = query_interval.poll() {
                // pick a random node target
                let target_random_node_id = enr::NodeId::random();
                //println!("Connected Peers: {}", swarm.connected_peers());

                for enr in swarm.enr_entries() {
                    let node_id = enr.node_id().clone().to_string();
                    
                    if !peers.contains_key(&node_id){
                        let peer_id = enr.peer_id().clone().to_string();
                        let multiaddr: String = enr.multiaddr().iter().map(|m| m.to_string()+"    ").collect();;
                        peers.insert(node_id.clone(),(peer_id.clone(), multiaddr.clone()));
                        //let eth2 = enr.get("eth2");
                        println!("{0: <6}{1: <14}{2: <55}{3: <66}{4: }", peers.len(),node_id,peer_id,multiaddr,enr.to_base64());
                    }
                }
                //println!("Searching for peers...");
                // execute a FINDNODE query
                swarm.find_node(target_random_node_id);
            }

            match swarm.poll().expect("Error while polling swarm") {
                Async::Ready(Some(event)) => match event {
                    Discv5Event::FindNodeResult { key, closer_peers } => {
                       /* if !closer_peers.is_empty() {
                            println!("Query Completed. Nodes found:");
                            for n in closer_peers {
                                println!("Node: {}", n);
                            }
                        } else {
                            println!("Query Completed. No peers found.")
                        }*/
                    },
                    Discv5Event::Discovered (peer) => {
                        //let peer_id = peer.peer_id().to_string();
                        //let node_id =  peer.node_id().to_string();
                        //let multiaddr = peer.multiaddr();
                        //swarm.add_enr(peer);
                        //let eth2 = enr.get("eth2");
                        //println!("{0: <6}{1: <55}{2: <14}{3:?}", i, peer_id, node_id, multiaddr);
                    },
                    Discv5Event::EnrAdded { enr, replaced } => {
                        //println!("enr added");
                        //let peer_id = enr.peer_id().to_string();
                        //let node_id =  enr.node_id().to_string();
                        //let multiaddr = enr.multiaddr();
                        //let eth2 = enr.get("eth2");
                        //println!("{0: <55}{1: <14}{2:?}", peer_id, node_id, multiaddr);
                    }
                    _ => (),
                },
                Async::Ready(None) | Async::NotReady => break,
            }
        }

        Ok(Async::NotReady)
    }));
}
