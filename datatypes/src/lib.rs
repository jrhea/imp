use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::{runtime, signal, task, time};

pub enum Message {
    /// Message originating from command and control
    Command,
    /// Message originating from the network service
    Network,
    /// Message that tells services to shutdown
    Shutdown,
}
