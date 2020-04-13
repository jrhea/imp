extern crate target_info;
#[macro_use]
extern crate error_chain;
use clap::{App, AppSettings, Arg, ArgMatches};
use futures::executor::block_on;
use slog::{debug, info, o, warn, Drain};
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Weak;
use target_info::Target;

use std::time::Duration;
use tokio::process::Command;
use tokio::task::JoinHandle;
use tokio::{runtime, signal, sync, task, time};

use agent::Agent;
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

    let p2p_service = P2PService::new(client_name, platform, protocol_version, &arg_matches);
    let network_service = NetworkService::new(p2p_service.clone());
    let agent = Agent::new(network_service.clone());
    let mut runtime = runtime::Runtime::new()?;

    runtime.block_on(async move {
        task::spawn(async move {
            runner(p2p_service, network_service, agent).await;
        });
        signal::ctrl_c().await.expect("failed to listen for event");
    });

    println!("Shutting down");
    runtime.shutdown_timeout(Duration::from_millis(1000));
    Ok(())
}

async fn runner(
    p2p_service: Arc<P2PService>,
    network_service: Arc<NetworkService>,
    agent: Arc<Agent>,
) -> Result<(), std::io::Error> {
    let mut tasks = vec![];

    tasks.push(task::spawn(async move { p2p_service.spawn().await }));
    tasks.push(task::spawn(async move { network_service.spawn().await }));
    tasks.push(task::spawn(async move { agent.spawn().await }));

    for task in tasks {
        task.await??;
    }
    Ok(())
}
