use clap::ArgMatches;
use std::sync::Arc;
use std::{thread, time};
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot};

use crate::types::topics::create_topics;
use eth2_libp2p::types::{GossipEncoding, GossipKind, GossipTopic};
#[cfg(feature = "local")]
use eth2_libp2p_local as eth2_libp2p;
use mothra::{
    cli_app, config::Config, gossip, rpc_request, rpc_response, Mothra, NetworkGlobals,
    NetworkMessage,
};
#[cfg(feature = "local")]
use mothra_local as mothra;
use crate::types::FORK_DIGEST;

// Holds variables needed to interacts with mothra
pub struct Service {
    runtime: Runtime,
    network_globals: Arc<NetworkGlobals>,
    network_send: mpsc::UnboundedSender<NetworkMessage>,
    pub network_exit: Arc<oneshot::Sender<()>>,
    log: slog::Logger,
}

impl Service {
    pub fn new(
        client_name: String,
        platform: String,
        protocol_version: String,
        arg_matches: &ArgMatches<'_>,
    ) -> Arc<Self> {
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
        Arc::new(Service {
            runtime,
            network_globals,
            network_send,
            network_exit: Arc::new(network_exit),
            log,
        })
    }

    pub async fn spawn(&self) -> Result<(), std::io::Error> {
        let start = time::Instant::now();
        let dur = time::Duration::from_secs(5);
        loop {
            thread::sleep(dur);
            let data = format!("Hello from imp.  Elapsed time: {:?}", start.elapsed())
                .as_bytes()
                .to_vec();
            gossip(
                self.network_send.clone(),
                GossipTopic::new(
                    GossipKind::BeaconBlock,
                    GossipEncoding::default(),
                    FORK_DIGEST,
                )
                .into(),
                data,
                self.log.clone(),
            );
        }
        Ok(()) as Result<(), std::io::Error>
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
