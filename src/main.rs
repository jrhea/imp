extern crate target_info;
#[macro_use]
extern crate error_chain;
use clap::{App, AppSettings, Arg, ArgMatches};
use futures::executor::block_on;
use slog::{debug, info, o, warn, Drain};
use std::cell::RefCell;
use std::sync::{Arc,Mutex};
use std::time::Duration;
use tokio::{runtime, signal, sync, task, time};
use tokio::process::Command;
use tokio::task::JoinHandle;
use tokio::sync::oneshot;
use futures::future::{Abortable, AbortHandle, Aborted};

use agent::Agent;
use network::NetworkService;
use p2p::{cli_app, P2PService};
use datatypes::Env;

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
    let network_service = NetworkService::new(client_name, platform, protocol_version, &arg_matches);
    let agent = Agent::new(network_service.clone());
    let (network_shutdown_tx, network_shutdown_rx) = oneshot::channel::<()>();
    let (agent_shutdown_tx, agent_shutdown_rx) = oneshot::channel::<()>();

    runtime.block_on(async move {
        
        async move {
            network_service.spawn(network_shutdown_rx).await;
            agent.spawn(agent_shutdown_rx).await;
        }.await;
        // block the current thread until Ctrl+C is received.
        signal::ctrl_c().await.expect("failed to listen for event");       

    });
    println!("Sending shutdown signal.");
    network_shutdown_tx.send(()); 
    agent_shutdown_tx.send(()); 
    runtime.shutdown_timeout(Duration::from_millis(1000));
    println!("Exiting imp.");
    

    Ok(())
}
