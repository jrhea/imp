use std::sync::Arc;
use std::time::Duration;
use tokio::{runtime, time, task, signal};
use tokio::sync::oneshot;
use futures::future::{Abortable, AbortHandle, Aborted};
use network::{NetworkService};

pub struct Agent {
    network_service: Arc<NetworkService>,
}

impl Agent {
    pub fn new(network_service: Arc<NetworkService>) -> Arc<Self> {
        Arc::new(
            Agent {
                network_service
            }
        )
    }

    pub async fn spawn(&self, rx: oneshot::Receiver<()>) {
        task::spawn(async move {          
            if let Ok(()) = rx.await {
                println!("Agent: shutdown signal received.");
            } 
        });
    }
}