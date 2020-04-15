use clap::ArgMatches;
use std::any::type_name;
use std::sync::Arc;

use crate::types::topics::create_topics;
use crate::types::FORK_DIGEST;
use datatypes::Message;

#[cfg(not(feature = "local"))]
use mothra::{Mothra, NetworkGlobals, NetworkMessage};
#[cfg(feature = "local")]
use mothra_local::{Mothra, NetworkGlobals, NetworkMessage};

// Holds variables needed to interacts with mothra
pub struct Service {
    network_globals: Arc<NetworkGlobals>,
    network_send: tokio_01::sync::mpsc::UnboundedSender<NetworkMessage>,
    network_exit: tokio_01::sync::oneshot::Sender<()>,
    log: slog::Logger,
}

impl Service {
    pub fn new(
        executor: &tokio_compat::runtime::TaskExecutor,
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

        let (network_globals, network_send, network_exit, log) = Mothra::new(
            config,
            &executor,
            on_discovered_peer,
            on_receive_gossip,
            on_receive_rpc,
        )
        .unwrap();

        Service {
            network_globals,
            network_send,
            network_exit,
            log,
        }
    }

    pub async fn spawn(self, mut shutdown_rx: tokio_02::sync::watch::Receiver<Message>) {
        loop {
            match shutdown_rx.recv().await {
                Some(Message::Shutdown) => {
                    println!("{:?}: shutdown message received.", type_name::<Service>());
                    self.network_exit.send(());
                    break;
                }
                _ => (),
            };
        }
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
