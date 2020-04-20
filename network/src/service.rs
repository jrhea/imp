use slog::{debug, info, o, trace, warn};
use std::any::type_name;
use tokio::sync::watch;
use tokio::{runtime, signal, task, time};
use types::events::Events;
use p2p::P2PAdapter;

pub struct Service {
    p2p_adapter: P2PAdapter,
    log: slog::Logger,
}

impl Service {
    pub fn new(p2p_adapter: P2PAdapter, log: slog::Logger) -> Self {
        Service { 
            p2p_adapter,
            log 
        }
    }
    pub async fn spawn(self, mut shutdown_rx: watch::Receiver<Events>) {
        task::spawn(async move {
            loop {
                match shutdown_rx.recv().await {
                    Some(Events::ShutdownMessage) => {
                        info!(
                            self.log,
                            "{:?}: shutdown message received.",
                            type_name::<Service>()
                        );
                        let _ = self.p2p_adapter.close();
                        break;
                    }
                    _ => (),
                };
            }
        });
    }
}
