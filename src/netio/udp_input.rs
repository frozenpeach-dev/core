//! Simple [`Input`] implementation using the
//! UDP protocol. It reads bytes from a [`UdpSocket`]
//! and turns them into a [`PacketType`] implementation
//! by calling `from_raw_bytes`

use std::io;

use async_trait::async_trait;
use tokio::net::UdpSocket;

use crate::core::{packet::PacketType, state_switcher::Input};

/// `UdpInput` provides a simple implementation of
/// an [`Input`] using the UDP protocol.
pub struct UdpInput {
    socket: UdpSocket,
}

impl UdpInput {
    /// Binds the `UdpInput` listener to the provided address
    ///
    /// # Examples:
    ///
    /// ```
    /// let udp_input = UdpInput::start("0.0.0.0:53");
    /// ```
    pub async fn start(addr: &str) -> Result<Self, std::io::Error> {
        Ok(Self {
            socket: UdpSocket::bind(addr).await?,
        })
    }

    /// Returns the next message received
    async fn get_next(&self) -> Result<Vec<u8>, io::Error> {
        let mut buf = [0u8; 65535];
        let (bytes_len, src_addr) = self.socket.recv_from(&mut buf).await?;

        Ok(buf[..bytes_len].to_vec())
    }
}

#[async_trait]
impl<T: PacketType> Input<T> for UdpInput {
    async fn get(&self) -> Result<T, io::Error> {
        let buf = self.get_next().await?;
        Ok(T::from_raw_bytes(&buf))
    }
}
