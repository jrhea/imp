extern crate target_info;
#[macro_use]
extern crate error_chain;
use clap::{App, AppSettings, Arg, ArgMatches};
use futures::executor::block_on;
use futures::future::{AbortHandle, Abortable, Aborted};
use slog::{debug, info, o, warn, Drain};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::{runtime, signal, sync, task, time};

use agent::Agent;
use datatypes::Message;
use network::NetworkService;
use p2p::{cli_app, P2PService};

const CLIENT_NAME: &str = "imp";
const PROTOCOL_VERSION: &str = "imp/libp2p";

fn main() -> Result<(), std::io::Error> {
    // Parse the CLI parameters.
    let arg_matches = App::new(CLIENT_NAME)
        .version(clap::crate_version!())
        .author("Jonny Rhea")
        .about("Eth2 Network Agent")
        .subcommand(cli_app())
        .get_matches();

    let client_name: String = CLIENT_NAME.into();
    let platform: String = format!("v{}", env!("CARGO_PKG_VERSION"));
    let protocol_version: String = PROTOCOL_VERSION.into();

    let mut runtime = runtime::Runtime::new()?;
    let mut p2p_service = P2PService::new(client_name, platform, protocol_version, &arg_matches);
    let network_service = NetworkService::new();
    let agent = Agent::new(network_service.clone());
    let (p2p_tx, p2p_rx) = mpsc::unbounded_channel::<Message>();
    let (network_tx, network_rx) = mpsc::unbounded_channel::<Message>();
    let (agent_tx, agent_rx) = mpsc::unbounded_channel::<Message>();

    runtime.block_on(async move {
        async move {
            task::spawn(async move {
                p2p_service.spawn(p2p_rx).await;
            });
            network_service.spawn(network_rx).await;
            agent.spawn(agent_rx).await;
        }
        .await;
        // block the current thread until Ctrl+C is received.
        signal::ctrl_c().await.expect("failed to listen for event");
    });
    println!("Sending shutdown signal.");
    p2p_tx.send(Message::Shutdown);
    network_tx.send(Message::Shutdown);
    agent_tx.send(Message::Shutdown);
    runtime.shutdown_timeout(Duration::from_millis(1000));
    println!("Exiting imp.");

    Ok(())
}
