extern crate target_info;
use clap::App;
use slog::{debug, info, o, warn, Drain};
use std::{thread, time};
use tokio_compat::runtime::Runtime;

#[cfg(feature = "local")]
use eth2_libp2p_local as eth2_libp2p;
#[cfg(feature = "local")]
use mothra_local as mothra;

use eth2_libp2p::types::{GossipEncoding, GossipKind, GossipTopic};
use mothra::{cli_app, gossip, Mothra};

use p2p::{topics, FORK_DIGEST};

const CLIENT_NAME: &str = "mock-node";
const PROTOCOL_VERSION: &str = "imp/libp2p";

fn main() {
    let start = time::Instant::now();
    // Parse the CLI parameters.
    let matches = App::new(CLIENT_NAME)
        .version(clap::crate_version!())
        .author("Jonny Rhea")
        .about("Eth2 mock node")
        .subcommand(cli_app())
        .get_matches();

    let runtime = Runtime::new()
        .map_err(|e| format!("Failed to start runtime: {:?}", e))
        .unwrap();
    let executor = runtime.executor();

    let mut config = Mothra::get_config(
        Some(CLIENT_NAME.into()),
        Some(format!("v{}", env!("CARGO_PKG_VERSION"))),
        Some(PROTOCOL_VERSION.into()),
        &matches.subcommand_matches("mothra").unwrap(),
    );

    config.network_config.topics = topics::create_topics(FORK_DIGEST);

    let (network_globals, network_send, network_exit, network_logger) = Mothra::new(
        config,
        &executor,
        on_discovered_peer,
        on_receive_gossip,
        on_receive_rpc,
    )
    .unwrap();

    let dur = time::Duration::from_secs(5);
    loop {
        thread::sleep(dur);
        let data = format!("Hello from mock-node.  Elapsed time: {:?}", start.elapsed())
            .as_bytes()
            .to_vec();
        gossip(
            network_send.clone(),
            GossipTopic::new(
                GossipKind::BeaconBlock,
                GossipEncoding::default(),
                FORK_DIGEST,
            )
            .into(),
            data,
            network_logger.clone(),
        );
    }
}

fn on_discovered_peer(peer: String) {
    println!("{}: discovered peer", CLIENT_NAME);
    println!("peer={:?}", peer);
}

fn on_receive_gossip(topic: String, data: Vec<u8>) {
    println!("{}: received gossip", CLIENT_NAME);
    println!("topic={:?}", topic);
    println!("data={:?}", String::from_utf8_lossy(&data));
}

fn on_receive_rpc(method: String, req_resp: u8, peer: String, data: Vec<u8>) {
    println!("{}: received rpc", CLIENT_NAME);
    println!("method={:?}", method);
    println!("req_resp={:?}", req_resp);
    println!("peer={:?}", peer);
    println!("data={:?}", String::from_utf8_lossy(&data));
}
