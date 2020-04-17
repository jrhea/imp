use network::NetworkService;
use slog::{debug, info, o, trace, warn};
use std::any::type_name;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task;
use types::events::Events;

pub struct Agent {
    log: slog::Logger,
}

impl Agent {
    pub fn new(log: slog::Logger) -> Self {
        Agent { log }
    }

    pub async fn spawn(self, mut shutdown_rx: watch::Receiver<Events>) {
        task::spawn(async move {
            loop {
                match shutdown_rx.recv().await {
                    Some(Events::ShutdownMessage) => {
                        info!(
                            self.log,
                            "{:?}: shutdown message received.",
                            type_name::<Agent>()
                        );
                        break;
                    }
                    _ => (),
                };
            }
        });
    }
}
