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

pub struct Service {
    p2p_service: Arc<P2PService>,
}

impl Service {
    pub fn new(
        client_name: String,
        platform: String,
        protocol_version: String,
        arg_matches: &ArgMatches<'_>,
    ) -> Arc<Self> {
        Arc::new(Service {
            p2p_service: P2PService::new(client_name, platform, protocol_version, &arg_matches),
        })
    }
    pub async fn spawn(&self, mut rx: mpsc::UnboundedReceiver<Message>) {
        let p2p_service = self.p2p_service.clone();
        task::spawn(async move { p2p_service.spawn().await });
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
