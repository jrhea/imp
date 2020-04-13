use std::time::Duration;
use tokio::time;
use std::sync::Arc;
use std::sync::Weak;

use p2p::P2PService;

pub struct Service {
    p2p_service: Arc<P2PService>,
}

impl Service {
    pub fn new(p2p_service: Arc<P2PService>) -> Arc<Self> {
        Arc::new(
            Service {
                p2p_service
            }
        )
    }

    pub async fn spawn(&self) -> Result<(), std::io::Error> {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            println!("Network Service is awake.")
        }
        Ok(()) as Result<(), std::io::Error>
    }
}