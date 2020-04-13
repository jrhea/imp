use std::sync::Arc;
use std::time::Duration;
use tokio::time;

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

    pub async fn spawn(&self) -> Result<(), std::io::Error> {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            println!("Agent is awake.")
        }
        Ok(()) as Result<(), std::io::Error>
    }
}