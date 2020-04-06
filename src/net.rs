// exports identifiers from private sub-modules in the current module namespace
pub use self::socket::Peer;
pub use self::connection::Connection;
pub use self::socket::Socket;

mod socket;
mod connection;
mod connection_manager;

pub mod constants;
