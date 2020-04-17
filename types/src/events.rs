#[derive(Clone, Copy)]
pub enum Events {
    /// Message originating from command and control
    CommandMessage,
    /// Message originating from the network service
    NetworkMessage,
    /// Message that tells services to shutdown
    ShutdownMessage,
    /// No value
    None,
}