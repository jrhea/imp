use datatypes::Message;
use network::NetworkService;
use std::any::type_name;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task;

pub struct Agent {
    network_service: Arc<NetworkService>,
}

impl Agent {
    pub fn new(network_service: Arc<NetworkService>) -> Arc<Self> {
        Arc::new(Agent { network_service })
    }

    pub async fn spawn(&self, mut shutdown_rx: watch::Receiver<Message>) {
        task::spawn(async move {
            loop {
                match shutdown_rx.recv().await {
                    Some(Message::Shutdown) => {
                        println!("{:?}: shutdown message received.", type_name::<Agent>());
                        break;
                    }
                    _ => (),
                };
            };
        });
    }
}
