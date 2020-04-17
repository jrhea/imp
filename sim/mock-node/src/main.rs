extern crate target_info;
use clap::App;
use eth2::libp2p::types::GossipKind;
use p2p::mothra::{cli_app, gossip, Mothra};
use slog::{debug, info, o, trace, warn};
use std::path::PathBuf;
use std::{thread, time};
use tokio_compat::runtime::Runtime;

use eth2::ssz::Encode;

const CLIENT_NAME: &str = "mock-node";
const PROTOCOL_VERSION: &str = "imp/libp2p";

fn main() {
    let start = time::Instant::now();
    // Parse the CLI parameters.
    let arg_matches = App::new(CLIENT_NAME)
        .version(clap::crate_version!())
        .author("Jonny Rhea")
        .about("Eth2 mock node")
        .subcommand(cli_app())
        .get_matches();

    // get mothra subcommand args matches
    let mothra_arg_matches = &arg_matches.subcommand_matches("mothra").unwrap();

    let debug_level = mothra_arg_matches.value_of("debug-level").unwrap();

    // configure logging
    let slog = utils::config_logger(debug_level, true);
    let log = slog.new(o!("mock-node" => "mock-node"));

    let mut config = Mothra::get_config(
        Some(CLIENT_NAME.into()),
        Some(format!("v{}", env!("CARGO_PKG_VERSION"))),
        Some(PROTOCOL_VERSION.into()),
        &mothra_arg_matches,
    );

    let testnet_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".lighthouse")
        .join("testnet");

    let enr_fork_id = eth2::utils::get_genesis_enr_fork_id(Some(testnet_dir));
    config.network_config.topics = eth2::utils::create_topic_ids(enr_fork_id.clone());

    let runtime = Runtime::new()
        .map_err(|e| format!("Failed to start runtime: {:?}", e))
        .unwrap();
    let executor = runtime.executor();

    let (network_globals, network_send, network_exit) = Mothra::new(
        config,
        enr_fork_id.clone().as_ssz_bytes(),
        &executor,
        on_discovered_peer,
        on_receive_gossip,
        on_receive_rpc,
        log.new(o!("mock-node" => "Mothra")),
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
            eth2::utils::get_gossip_topic_id(GossipKind::BeaconBlock, enr_fork_id.clone()),
            data,
            log.new(o!("mock-node" => "Mothra")),
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
