use clap::ArgMatches;
use discovery::Discovery;
use p2p::{discovery, P2PAdapter};
use slog::{debug, info, o, trace, warn};
use std::any::type_name;
use std::path::PathBuf;
use tokio_02::sync::watch;
use tokio_02::{signal, task, time};
use types::events::Events;

pub struct Service {
    run_mode: String,
    p2p_adapter: Option<P2PAdapter>,
    discovery: Discovery,
    log: slog::Logger,
}

impl Service {
    pub fn new(
        run_mode: String,
        executor: &tokio_compat::runtime::TaskExecutor,
        client_name: String,
        platform: String,
        p2p_protocol_version: String,
        testnet_dir: Option<PathBuf>,
        arg_matches: &ArgMatches<'_>,
        log: slog::Logger,
    ) -> Self {
        let (p2p_adapter, discovery) = match run_mode.as_str() {
            "node" => (
                Some(P2PAdapter::new(
                    &executor,
                    client_name,
                    platform,
                    p2p_protocol_version,
                    testnet_dir,
                    &arg_matches,
                    log.new(o!("NetworkService" => "P2PAdapter")),
                )),
                (None, None, None),
            ),
            "disc" => (
                None,
                discovery::init(arg_matches, log.new(o!("Network Service" => "Discovery"))),
            ),
            _ => (None, (None, None, None)),
        };

        Service {
            run_mode,
            p2p_adapter,
            discovery,
            log,
        }
    }
    pub async fn spawn(self, mut shutdown_rx: watch::Receiver<Events>) {
        let run_mode = self.run_mode;
        let p2p_adapter = self.p2p_adapter;
        let discovery = self.discovery;
        let disc_log = self.log.clone();
        let service_log = self.log.clone();
        let (swarm, discovery_shutdown_tx, discovery_shutdown_rx) = match run_mode.as_str() {
            "node" => (None, None, None),
            "disc" => discovery,
            _ => (None, None, None),
        };
        task::spawn(async move {
            if let "disc" = run_mode.as_str() {
                task::spawn(async move {
                    discovery::find_nodes(
                        swarm.unwrap(),
                        discovery_shutdown_rx.unwrap(),
                        disc_log.new(o!("Network Service" => "Discovery")),
                    )
                    .await;
                });
            }

            loop {
                if let Some(Events::ShutdownMessage) = shutdown_rx.recv().await {
                    warn!(
                        service_log,
                        "{:?}: shutdown message received.",
                        type_name::<Service>()
                    );
                    break;
                }
            }

            match run_mode.as_str() {
                "node" => {
                    let _ = match p2p_adapter {
                        Some(p2p_adapter) => p2p_adapter.close(),
                        None => Err(()),
                    };
                }
                "disc" => {
                    let _ = match discovery_shutdown_tx {
                        Some(discovery_shutdown_tx) => discovery_shutdown_tx.send(()),
                        None => Err(()),
                    };
                }
                _ => (),
            };
        });
    }
}
