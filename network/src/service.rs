use clap::ArgMatches;
use p2p::{discovery::discover_peers, P2PAdapter};
use slog::{debug, info, o, trace, warn};
use std::any::type_name;
use std::path::PathBuf;
use tokio::sync::watch;
use tokio::{signal, task, time};
use types::events::Events;

pub struct Service {
    p2p_adapter: Option<P2PAdapter>,
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
        let p2p_adapter = match run_mode.as_str() {
            "node" => Some(P2PAdapter::new(
                &executor,
                client_name,
                platform,
                p2p_protocol_version,
                testnet_dir,
                &arg_matches,
                log.new(o!("NetworkService" => "P2PAdapter")),
            )),
            "disc" => {
                discover_peers(arg_matches);
                None
            }
            _ => None,
        };

        Service { p2p_adapter, log }
    }
    pub async fn spawn(self, mut shutdown_rx: watch::Receiver<Events>) {
        task::spawn(async move {
            loop {
                if let Some(Events::ShutdownMessage) = shutdown_rx.recv().await {
                    info!(
                    self.log,
                    "{:?}: shutdown message received.",
                    type_name::<Service>()
                    );
                    break;
                }
            }
            let _ = match self.p2p_adapter {
                Some(p2p_adapter) => p2p_adapter.close(),
                None => Err(()),
            };
        });
    }
}
