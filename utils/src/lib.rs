use clap::{App, Arg};
use env_logger::Env;
use slog::{debug, info, o, trace, warn, Drain, Level, Logger};

pub fn config_logger(debug_level: &str, init: bool) -> slog::Logger {
    if init {
        env_logger::Builder::from_env(Env::default()).init();
    }
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
    Logger::root(drain.fuse(), o!())
}
