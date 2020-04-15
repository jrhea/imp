extern crate target_info;
#[macro_use]
extern crate error_chain;
use clap::{App, Arg};
use env_logger::Env;
use slog::{debug, info, o, trace, warn, Drain, Level, Logger};
use std::time::Duration;
use tokio_01::prelude::future::Future;
use tokio_02::sync::watch;
use tokio_02::{signal, task, time};

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

    let debug_level = arg_matches
        .value_of("debug-level")
        .ok_or_else(|| "Expected --debug-level flag".to_string())
        .unwrap();

    // configure logging
    env_logger::Builder::from_env(Env::default()).init();
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build();
    let drain = match debug_level {
        "info" => drain.filter_level(Level::Info),
        "debug" => drain.filter_level(Level::Debug),
        "trace" => drain.filter_level(Level::Trace),
        "warn" => drain.filter_level(Level::Warning),
        "error" => drain.filter_level(Level::Error),
        "crit" => drain.filter_level(Level::Critical),
        _ => drain.filter_level(Level::Info),
    };
    let slog = Logger::root(drain.fuse(), o!());
    let log = slog.new(o!("imp" => "imp"));

    let client_name: String = CLIENT_NAME.into();
    let platform: String = format!("v{}", env!("CARGO_PKG_VERSION"));
    let protocol_version: String = PROTOCOL_VERSION.into();

    let mut runtime = tokio_compat::runtime::Runtime::new()?;
    let p2p_service = P2PService::new(
        &runtime.executor(),
        client_name,
        platform,
        protocol_version,
        &arg_matches,
        log.new(o!("imp" => "P2PService")),
    );
    let network_service = NetworkService::new();
    let agent = Agent::new(network_service.clone());

    runtime.block_on_std(async move {
        let (shutdown_tx, shutdown_rx) = watch::channel::<Message>(Message::None);
        async move {
            let rx = shutdown_rx.clone();
            task::spawn(async move {
                p2p_service.spawn(rx).await;
            });
            network_service.spawn(shutdown_rx.clone()).await;
            agent.spawn(shutdown_rx).await;
        }
        .await;
        // block the current thread until Ctrl+C is received.
        signal::ctrl_c().await.expect("failed to listen for event");
        println!("Sending shutdown signal.");
        let _ = shutdown_tx.broadcast(Message::Shutdown);
    });

    // Shutdown the runtime
    runtime.shutdown_on_idle().wait().unwrap();

    println!("Exiting imp.");

    Ok(())
}
