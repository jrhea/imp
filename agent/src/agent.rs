use datatypes::Message;
use futures::future::{AbortHandle, Abortable, Aborted};
use network::NetworkService;
use std::any::type_name;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::{runtime, signal, task, time};

pub struct Agent {
    network_service: Arc<NetworkService>,
}

impl Agent {
    pub fn new(network_service: Arc<NetworkService>) -> Arc<Self> {
        Arc::new(Agent { network_service })
    }

    pub async fn spawn(&self, mut rx: mpsc::UnboundedReceiver<Message>) {
        task::spawn(async move {
            match rx.recv().await {
                Some(Message::Command) => {
                    println!("{:?}: command message received.", type_name::<Agent>());
                }
                Some(Message::Network) => {
                    println!("{:?}: network message received.", type_name::<Agent>());
                }
                Some(Message::Shutdown) => {
                    println!("{:?}: shutdown message received.", type_name::<Agent>());
                }
                _ => (),
            }
        });
    }
}
