use std::time::Duration;
use std::sync::Arc;
use tokio::{runtime, time, task, signal};
use tokio::sync::oneshot;
use futures::future::{Abortable, AbortHandle, Aborted, TryFutureExt};
use clap::ArgMatches;
use futures::future;
use datatypes::Env;


use p2p::P2PService;

pub struct Service {
    p2p_service: Arc<P2PService>,
}

impl Service {
    pub fn new(client_name: String, platform: String, protocol_version: String, arg_matches: &ArgMatches<'_>) -> Arc<Self> {
        Arc::new(
            Service {
                p2p_service: P2PService::new(client_name, platform, protocol_version, &arg_matches),
            }
        )
    }
pub async fn spawn(&self, rx: oneshot::Receiver<()>) {
        let p2p_service = self.p2p_service.clone();
        task::spawn(async move { p2p_service.spawn().await });
        task::spawn(async move {          
            if let Ok(()) = rx.await {
                println!("Network Service: shutdown signal received.");
            } 
        });
    }
}