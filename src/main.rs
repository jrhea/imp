
extern crate target_info;
use target_info::Target;
use std::{thread, time};
use tokio::runtime::Runtime;
use slog::{debug, info, o, warn, Drain};

#[cfg(feature = "local")]
use mothra_local as mothra;
#[cfg(feature = "local")]
use eth2_libp2p_local as eth2_libp2p;

use mothra::{gossip, rpc_request, rpc_response, Mothra, NetworkMessage};
use eth2_libp2p::types::{GossipKind, GossipTopic, GossipEncoding};


const NAME: &str = "imp";
const PROTOCOL_VERSION: &str = "imp/libp2p";

fn main() {
    let start = time::Instant::now();
    let mut args: Vec<String> = std::env::args().collect();
    let runtime = Runtime::new()
        .map_err(|e| format!("Failed to start runtime: {:?}", e))
        .unwrap();
    let executor = runtime.executor();

    // Build Eth2 topics strings
    let fork_digest: [u8; 4] = [0; 4];
    let beacon_block_topic: String = GossipTopic::new(GossipKind::BeaconBlock,GossipEncoding::default(),fork_digest).into();
    let beacon_aggregate_and_proof_topic: String = GossipTopic::new(GossipKind::BeaconAggregateAndProof,GossipEncoding::default(),fork_digest).into();
    let voluntary_exit_topic: String = GossipTopic::new(GossipKind::VoluntaryExit,GossipEncoding::default(),fork_digest).into();
    let proposer_slashing_topic: String = GossipTopic::new(GossipKind::ProposerSlashing,GossipEncoding::default(),fork_digest).into();
    let attester_slashing_topic: String = GossipTopic::new(GossipKind::AttesterSlashing,GossipEncoding::default(),fork_digest).into();
    let topics: String = format!("{},{},{},{},{}",beacon_block_topic,beacon_aggregate_and_proof_topic,voluntary_exit_topic,proposer_slashing_topic,attester_slashing_topic);
    
    // Add the topics to the config args
    args.push("--topics".into());
    args.push(topics);

    let (network_globals, network_send, network_exit, network_logger) = Mothra::new(
            Some(NAME.into()),
            Some(format!("v{}",env!("CARGO_PKG_VERSION"))),
            Some(PROTOCOL_VERSION.into()),
            args,
            &executor,
            on_discovered_peer,
            on_receive_gossip,
            on_receive_rpc,
    ).unwrap();

    let dur = time::Duration::from_secs(5);
    loop {
        thread::sleep(dur);
        let data = format!("Hello from imp.  Elapsed time: {:?}",start.elapsed()).as_bytes().to_vec();
        gossip(network_send.clone(),beacon_block_topic.clone(),data,network_logger.clone());
    }
}


fn on_discovered_peer (peer: String){
    println!("{}: discovered peer",NAME);
    println!("peer={:?}", peer);
}

fn on_receive_gossip (topic: String, data: Vec<u8>){
    println!("{}: received gossip",NAME);
    println!("topic={:?}", topic);
    println!("data={:?}", String::from_utf8_lossy(&data));
}

fn on_receive_rpc (method: String, req_resp: u8, peer: String, data: Vec<u8>) { 
    println!("{}: received rpc",NAME);
    println!("method={:?}", method);
    println!("req_resp={:?}", req_resp);
    println!("peer={:?}", peer);
    println!("data={:?}", String::from_utf8_lossy(&data));
}