extern crate error_chain;
extern crate target_info;
use clap::{App, Arg};
use slog::{debug, info, o, trace, warn};
use std::time::Duration;
use tokio_01::prelude::future::Future;
use tokio_02::sync::watch;
use tokio_02::{signal, task, time};

use agent::Agent;
use datatypes::Message;
use network::NetworkService;
use p2p::{cli_app, P2PService};

const CLIENT_NAME: &str = "imp";
const P2P_PROTOCOL_VERSION: &str = "imp/libp2p";

fn main() -> Result<(), std::io::Error> {
    // Parse the CLI parameters.
    let arg_matches = App::new(CLIENT_NAME)
        .version(clap::crate_version!())
        .author("Jonny Rhea")
        .about("Eth2 Network Agent")
        .arg(
            Arg::with_name("p2p-protocol-version")
                .long("p2p-protocol-version")
                .value_name("P2P_PROTOCOL_VERSION")
                .help("P2P protocol version to advertise.")
                .takes_value(true)
                .default_value(P2P_PROTOCOL_VERSION),
        )
        .arg(
            Arg::with_name("debug-level")
                .long("debug-level")
                .value_name("LEVEL")
                .help("Log filter.")
                .takes_value(true)
                .possible_values(&["info", "debug", "trace", "warn", "error", "crit"])
                .default_value("info"),
        )
        .subcommand(cli_app())
        .get_matches();

    let p2p_protocol_version = arg_matches.value_of("p2p-protocol-version").unwrap();

    // default to this debug-level value.
    // if mothra submommand has a specific debug-level,
    // then mothra will use it
    let debug_level = arg_matches.value_of("debug-level").unwrap();

    // configure logging
    let slog = utils::config_logger(debug_level, true);
    let log = slog.new(o!("imp" => ""));

    let client_name: String = CLIENT_NAME.into();
    let platform: String = format!("v{}", env!("CARGO_PKG_VERSION"));

    let mut runtime = tokio_compat::runtime::Runtime::new()?;

    info!(log, "Starting imp");

    let p2p_service = P2PService::new(
        &runtime.executor(),
        client_name,
        platform,
        p2p_protocol_version.into(),
        &arg_matches,
        log.new(o!("imp" => "P2PService")),
    );
    let network_service = NetworkService::new(log.new(o!("imp" => "NetworkService")));
    let agent = Agent::new(log.new(o!("imp" => "Agent")));

    let (shutdown_tx, shutdown_rx) = watch::channel::<Message>(Message::None);

    // main "event loop"
    runtime.block_on_std(async move {
        async move {
            let rx = shutdown_rx.clone();
            task::spawn(async move {
                p2p_service.spawn(rx).await;
            });
            network_service.spawn(shutdown_rx.clone()).await;
            agent.spawn(shutdown_rx).await;
        }
        .await;
        // block the current thread until SIGINT is received.
        signal::ctrl_c().await.expect("failed to listen for event");
    });

    info!(log, "Sending shutdown signal.");
    let _ = shutdown_tx.broadcast(Message::Shutdown);

    // Shutdown the runtime
    runtime.shutdown_on_idle().wait().unwrap();
    info!(log.clone(), "Exiting imp.");
    Ok(())
}
