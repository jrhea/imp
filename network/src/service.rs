use slog::{debug, info, o, trace, warn};
use std::any::type_name;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::{runtime, signal, task, time};
use types::events::Events;

pub struct Service {
    log: slog::Logger,
}

impl Service {
    pub fn new(log: slog::Logger) -> Self {
        Service { log }
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
                        break;
                    }
                    _ => (),
                };
            }
        });
    }
}
