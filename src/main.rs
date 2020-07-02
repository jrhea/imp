extern crate error_chain;
extern crate target_info;
use agent::Agent;
use clap::{App, Arg};
use network::NetworkService;
use p2p;
use slog::{debug, info, o, trace, warn};
use std::path::PathBuf;
use std::thread::sleep;
use tokio::sync::watch;
use tokio::{signal, time::timeout, time::Duration};
use types::events::Events;

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
            Arg::with_name("testnet-dir")
                .long("testnet-dir")
                .value_name("DIR")
                .help("The location of the testnet directory to use.")
                .takes_value(true),
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
        .subcommand(p2p::cli_app())
        .subcommand(p2p::crawler::cli_app())
        .get_matches();

    let p2p_protocol_version = arg_matches.value_of("p2p-protocol-version").unwrap();

    let mut testnet_dir = None;
    if let Some(testnet_dir_str) = arg_matches.value_of("testnet-dir") {
        testnet_dir = Some(PathBuf::from(testnet_dir_str));
    }

    // default to this debug-level value.
    // if mothra submommand has a specific debug-level,
    // then mothra will use it
    let debug_level = arg_matches.value_of("debug-level").unwrap();

    // configure logging
    let slog = utils::config_logger(debug_level, true);
    let log = slog.new(o!("imp" => ""));

    let client_name: String = CLIENT_NAME.into();
    let platform: String = format!("v{}", env!("CARGO_PKG_VERSION"));

    let mut runtime = tokio::runtime::Runtime::new()?;

    info!(log, "Starting imp");

    let network_service = NetworkService::new(
        &runtime,
        client_name,
        platform,
        p2p_protocol_version.into(),
        testnet_dir,
        &arg_matches,
        log.new(o!("imp" => "NetworkService")),
    );
    let agent = Agent::new(log.new(o!("imp" => "Agent")));

    let (shutdown_tx, shutdown_rx) = watch::channel::<Events>(Events::None);

    // main "event loop"
    runtime.block_on(async move {
        async move {
            network_service.spawn(shutdown_rx.clone()).await;
            agent.spawn(shutdown_rx).await;
        }
        .await;
        // block the current thread until SIGINT is received.
        signal::ctrl_c().await.expect("failed to listen for event");
    });

    warn!(log, "Sending shutdown signal.");
    let _ = shutdown_tx.broadcast(Events::ShutdownMessage);

    sleep(Duration::new(1, 0));

    // Shutdown the runtime
    let _ = runtime.shutdown_timeout(tokio::time::Duration::from_millis(300));

    warn!(log.clone(), "Exiting imp.");
    Ok(())
}
