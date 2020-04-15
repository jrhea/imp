extern crate target_info;
use clap::App;
use env_logger::Env;
use slog::{debug, info, o, trace, warn, Drain, Level, Logger};
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
    // configure logging
    env_logger::Builder::from_env(Env::default()).init();
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build();
    let drain = match config.debug_level.as_str() {
        "info" => drain.filter_level(Level::Info),
        "debug" => drain.filter_level(Level::Debug),
        "trace" => drain.filter_level(Level::Trace),
        "warn" => drain.filter_level(Level::Warning),
        "error" => drain.filter_level(Level::Error),
        "crit" => drain.filter_level(Level::Critical),
        _ => drain.filter_level(Level::Info),
    };
    let slog = Logger::root(drain.fuse(), o!());
    let log = slog.new(o!("mock-node" => "mock-node"));

    config.network_config.topics = topics::create_topics(FORK_DIGEST);

    let (network_globals, network_send, network_exit) = Mothra::new(
        config,
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
            GossipTopic::new(
                GossipKind::BeaconBlock,
                GossipEncoding::default(),
                FORK_DIGEST,
            )
            .into(),
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
