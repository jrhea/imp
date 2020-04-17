use clap::ArgMatches;
use slog::{debug, info, o, trace, warn};
use std::any::type_name;
use std::sync::Arc;

use eth2::utils::create_topic_ids;
use types::events::Events;

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
        log: slog::Logger,
    ) -> Self {
        let mut mothra_log = log.clone();

        // get mothra subcommand args matches
        let mothra_arg_matches = &arg_matches.subcommand_matches("mothra").unwrap();

        // if debug-level is set in subcommand
        // Note: bc mothra sets default value to info
        // mothra_arg_matches.is_present is always true
        // so we have to use mothra_arg_matches.occurrences_of
        if mothra_arg_matches.occurrences_of("debug-level") > 0 {
            let debug_level = mothra_arg_matches.value_of("debug-level").unwrap();

            // re-configure logging
            mothra_log = utils::config_logger(debug_level, false).new(o!("P2PService" => "Mothra"));
        }

        let mut config = Mothra::get_config(
            Some(client_name),
            Some(platform),
            Some(protocol_version),
            &mothra_arg_matches,
        );

        // configure gossip topics
        config.network_config.topics = create_topic_ids();
        // instantiate mothra
        let (network_globals, network_send, network_exit) = Mothra::new(
            config,
            &executor,
            on_discovered_peer,
            on_receive_gossip,
            on_receive_rpc,
            mothra_log,
        )
        .unwrap();

        Service {
            network_globals,
            network_send,
            network_exit,
            log,
        }
    }

    pub async fn spawn(self, mut shutdown_rx: tokio_02::sync::watch::Receiver<Events>) {
        loop {
            match shutdown_rx.recv().await {
                Some(Events::ShutdownMessage) => {
                    info!(
                        self.log,
                        "{:?}: shutdown message received.",
                        type_name::<Service>()
                    );
                    let _ = self.network_exit.send(());
                    break;
                }
                _ => (),
            };
        }
    }
}

fn on_discovered_peer(peer: String) {
    println!("new peer discovered");
    println!("peer: {:?}", peer);
}

fn on_receive_gossip(topic: String, data: Vec<u8>) {
    println!("gossip message received");
    println!("topic: {:?}", topic);
    println!("data: {:?}", String::from_utf8_lossy(&data));
}

fn on_receive_rpc(method: String, req_resp: u8, peer: String, data: Vec<u8>) {
    println!("rpc message received");
    println!("method: {:?}", method);
    println!("req_resp: {:?}", req_resp);
    println!("peer: {:?}", peer);
    println!("data: {:?}", String::from_utf8_lossy(&data));
}
