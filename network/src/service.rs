use datatypes::Message;
use std::any::type_name;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::{runtime, signal, task, time};

pub struct Service {}

impl Service {
    pub fn new() -> Arc<Self> {
        Arc::new(Service {})
    }
    pub async fn spawn(&self, mut shutdown_rx: watch::Receiver<Message>) {
        task::spawn(async move {
            loop {
                match shutdown_rx.recv().await {
                    Some(Message::Shutdown) => {
                        println!("{:?}: shutdown message received.", type_name::<Service>());
                        break;
                    }
                    _ => (),
                };
            }
        });
    }
}
