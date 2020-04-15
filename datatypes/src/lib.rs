#[derive(Clone, Copy)]
pub enum Message {
    /// Message originating from command and control
    Command,
    /// Message originating from the network service
    Network,
    /// Message that tells services to shutdown
    Shutdown,
    /// No value
    None,
}
