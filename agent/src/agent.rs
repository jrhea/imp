use datatypes::Message;
use network::NetworkService;
use slog::{debug, info, o, trace, warn};
use std::any::type_name;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task;

pub struct Agent {
    log: slog::Logger,
}

impl Agent {
    pub fn new(log: slog::Logger) -> Self {
        Agent { log }
    }

    pub async fn spawn(self, mut shutdown_rx: watch::Receiver<Message>) {
        task::spawn(async move {
            loop {
                match shutdown_rx.recv().await {
                    Some(Message::Shutdown) => {
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
