use clap::ArgMatches;
use crawler::Crawler;
use p2p::{crawler, P2PAdapter};
use slog::{debug, info, o, trace, warn};
use std::any::type_name;
use std::path::PathBuf;
use tokio_02::sync::watch;
use tokio_02::{signal, task, time};
use types::events::Events;

pub struct Service {
    run_mode: String,
    p2p_adapter: Option<P2PAdapter>,
    crawler: Crawler,
    log: slog::Logger,
}

impl Service {
    pub fn new(
        executor: &tokio_compat::runtime::TaskExecutor,
        client_name: String,
        platform: String,
        p2p_protocol_version: String,
        testnet_dir: Option<PathBuf>,
        arg_matches: &ArgMatches<'_>,
        log: slog::Logger,
    ) -> Self {
        let mut run_mode = "node";
        if let Some(matches) = arg_matches.subcommand_matches("crawler") {
            run_mode = "crawler";
        }

        let (p2p_adapter, crawler) = match run_mode {
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
                (None, None, None, None),
            ),
            "crawler" => (
                None,
                crawler::init(arg_matches, log.new(o!("Network Service" => "Discovery"))),
            ),
            _ => (None, (None, None, None, None)),
        };

        Service {
            run_mode: run_mode.into(),
            p2p_adapter,
            crawler,
            log,
        }
    }
    pub async fn spawn(self, mut shutdown_rx: watch::Receiver<Events>) {
        let run_mode = self.run_mode;
        let p2p_adapter = self.p2p_adapter;
        let crawler = self.crawler;
        let crawler_log = self.log.clone();
        let service_log = self.log.clone();
        let (swarm, crawler_shutdown_tx, crawler_shutdown_rx, datadir) = match run_mode.as_str() {
            "node" => (None, None, None, None),
            "crawler" => crawler,
            _ => (None, None, None, None),
        };
        task::spawn(async move {
            if let "crawler" = run_mode.as_str() {
                task::spawn(async move {
                    crawler::find_nodes(
                        swarm.unwrap(),
                        crawler_shutdown_rx.unwrap(),
                        datadir.unwrap(),
                        crawler_log.new(o!("Network Service" => "Crawler")),
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
                "crawler" => {
                    let _ = match crawler_shutdown_tx {
                        Some(crawler_shutdown_tx) => crawler_shutdown_tx.send(()),
                        None => Err(()),
                    };
                }
                _ => (),
            };
        });
    }
}
