extern crate target_info;
use clap::{App, AppSettings, Arg, ArgMatches};
use slog::{debug, info, o, warn, Drain};
use std::{thread, time};
use target_info::Target;
use tokio::runtime::Runtime;

#[cfg(feature = "local")]
use eth2_libp2p_local as eth2_libp2p;
#[cfg(feature = "local")]
use mothra_local as mothra;

use eth2_libp2p::types::{GossipEncoding, GossipKind, GossipTopic};
use mothra::{cli_app, config::Config, gossip, rpc_request, rpc_response, Mothra, NetworkMessage};

const CLIENT_NAME: &str = "imp";
const PROTOCOL_VERSION: &str = "imp/libp2p";

fn main() {
    let start = time::Instant::now();
    // Parse the CLI parameters.
    let matches = App::new("imp")
        .version(clap::crate_version!())
        .author("Jonny Rhea")
        .about("Eth2 Network Agent")
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

    // Build Eth2 topics strings
    let fork_digest: [u8; 4] = [0; 4];
    let beacon_block_topic: String = GossipTopic::new(
        GossipKind::BeaconBlock,
        GossipEncoding::default(),
        fork_digest,
    )
    .into();
    let beacon_aggregate_and_proof_topic: String = GossipTopic::new(
        GossipKind::BeaconAggregateAndProof,
        GossipEncoding::default(),
        fork_digest,
    )
    .into();
    let voluntary_exit_topic: String = GossipTopic::new(
        GossipKind::VoluntaryExit,
        GossipEncoding::default(),
        fork_digest,
    )
    .into();
    let proposer_slashing_topic: String = GossipTopic::new(
        GossipKind::ProposerSlashing,
        GossipEncoding::default(),
        fork_digest,
    )
    .into();
    let attester_slashing_topic: String = GossipTopic::new(
        GossipKind::AttesterSlashing,
        GossipEncoding::default(),
        fork_digest,
    )
    .into();

    config.network_config.topics = vec![
        beacon_block_topic.clone(),
        beacon_aggregate_and_proof_topic.clone(),
        voluntary_exit_topic.clone(),
        proposer_slashing_topic.clone(),
        attester_slashing_topic.clone(),
    ];

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
        let data = format!("Hello from imp.  Elapsed time: {:?}", start.elapsed())
            .as_bytes()
            .to_vec();
        gossip(
            network_send.clone(),
            beacon_block_topic.clone(),
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
