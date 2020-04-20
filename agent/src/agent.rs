use slog::{debug, info, o, trace, warn};
use std::any::type_name;
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
                if let Some(Events::ShutdownMessage) = shutdown_rx.recv().await {
                    info!(
                    self.log,
                    "{:?}: shutdown message received.",
                    type_name::<Agent>()
                    );
                }
            }
        });
    }
}
