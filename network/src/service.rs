use clap::ArgMatches;
use datatypes::Message;
use futures::future;
use futures::future::{AbortHandle, Abortable, Aborted, TryFutureExt};
use std::any::type_name;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::{runtime, signal, task, time};

use p2p::P2PService;

pub struct Service {}

impl Service {
    pub fn new() -> Arc<Self> {
        Arc::new(Service {})
    }
    pub async fn spawn(&self, mut rx: mpsc::UnboundedReceiver<Message>) {
        task::spawn(async move {
            match rx.recv().await {
                Some(Message::Command) => {
                    println!("{:?}: command message received.", type_name::<Service>());
                }
                Some(Message::Network) => {
                    println!("{:?}: network message received.", type_name::<Service>());
                }
                Some(Message::Shutdown) => {
                    println!("{:?}: shutdown message received.", type_name::<Service>());
                }
                _ => (),
            }
        });
    }
}
