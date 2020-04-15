use clap::ArgMatches;
use futures::{sync::oneshot, Future};
use std::any::type_name;
use std::cell::RefCell;
use std::sync::Arc;
use std::{thread, time};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use crate::types::topics::create_topics;
use crate::types::FORK_DIGEST;
use datatypes::Message;
use eth2_libp2p::types::{GossipEncoding, GossipKind, GossipTopic};
#[cfg(feature = "local")]
use eth2_libp2p_local as eth2_libp2p;
use mothra::{
    cli_app, config::Config, gossip, rpc_request, rpc_response, Mothra, NetworkGlobals,
    NetworkMessage,
};
#[cfg(feature = "local")]
use mothra_local as mothra;

// Holds variables needed to interacts with mothra
pub struct Service {
    runtime: Runtime,
    network_globals: Arc<NetworkGlobals>,
    network_send: mpsc::UnboundedSender<NetworkMessage>,
    pub network_exit: Arc<tokio::sync::oneshot::Sender<()>>,
    log: slog::Logger,
}

impl Service {
    pub fn new(
        client_name: String,
        platform: String,
        protocol_version: String,
        arg_matches: &ArgMatches<'_>,
    ) -> Self {
        let mut config = Mothra::get_config(
            Some(client_name),
            Some(platform),
            Some(protocol_version),
            &arg_matches.subcommand_matches("mothra").unwrap(),
        );

        config.network_config.topics = create_topics(FORK_DIGEST);

        let runtime = Runtime::new()
            .map_err(|e| format!("Failed to start runtime: {:?}", e))
            .unwrap();

        let (network_globals, network_send, network_exit, log) = Mothra::new(
            config,
            &runtime.executor(),
            on_discovered_peer,
            on_receive_gossip,
            on_receive_rpc,
        )
        .unwrap();
        Service {
            runtime,
            network_globals,
            network_send,
            network_exit: Arc::new(network_exit),
            log,
        }
    }

    pub async fn spawn(&mut self, mut rx: tokio2::sync::mpsc::UnboundedReceiver<Message>) {
        match rx.recv().await {
            Some(Message::Network) => {
                println!("{:?}: network message received.", type_name::<Service>());
            }
            Some(Message::Shutdown) => {
                println!("{:?}: shutdown message received.", type_name::<Service>());
            }
            _ => (),
        };
    }
}

fn on_discovered_peer(peer: String) {
    //println!("{}: discovered peer", CLIENT_NAME);
    println!("peer={:?}", peer);
}

fn on_receive_gossip(topic: String, data: Vec<u8>) {
    //println!("{}: received gossip", CLIENT_NAME);
    println!("topic={:?}", topic);
    println!("data={:?}", String::from_utf8_lossy(&data));
}

fn on_receive_rpc(method: String, req_resp: u8, peer: String, data: Vec<u8>) {
    //println!("{}: received rpc", CLIENT_NAME);
    println!("method={:?}", method);
    println!("req_resp={:?}", req_resp);
    println!("peer={:?}", peer);
    println!("data={:?}", String::from_utf8_lossy(&data));
}
