//! Simple [`Output`] implementation using the 
//! UDP protocol. It reads bytes from a [`PacketType`]
//! by calling `to_raw_bytes`, and turns these into
//! a UDP packet.
use std::net::{SocketAddrV4, Ipv4Addr};

use async_trait::async_trait;
use tokio::net::UdpSocket;

use crate::core::{state_switcher::Output, packet::PacketType};

/// `UdpOutput` provides a simple implementation of 
/// an [`Output`] using the UDP protocol.
pub struct UdpOutput{
    socket: UdpSocket,
}

impl UdpOutput {

    /// Binds the `UdpOutput` listener to the provided address
    ///
    /// # Examples:
    ///
    /// ```
    /// let udp_output = UdpInput::start("0.0.0.0:53");
    /// ```

    pub async fn start(addr: &str) -> Result<Self, std::io::Error> {
       Ok(Self { socket: UdpSocket::bind(addr).await? })
    }

}

#[async_trait]
impl<T: PacketType + Sync + Send + 'static> Output<T> for UdpOutput {
    /// Send a packet through the opened socket
    async fn send(&self, packet: T) -> Result<usize, std::io::Error> {
        let raw_bytes = packet.to_raw_bytes();
        if let Some(addr) = &raw_bytes.get(..6) {
            let addr = SocketAddrV4::new(Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]), ((addr[4] as u16) << 8) | addr[5] as u16);

            self.socket.send_to(&raw_bytes[6..], addr).await
        } else {
            return Ok(0)
        } 
        
    }
}
