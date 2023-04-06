use std::io;

use async_trait::async_trait;
use tokio::net::UdpSocket;

use crate::core::{state_switcher::Input, packet::PacketType};

pub struct UdpInput {
    socket: UdpSocket
} 

impl UdpInput {

    pub async fn start(addr: &str) -> Result<Self, std::io::Error> {
        Ok( Self{ socket: UdpSocket::bind(addr).await? } )
    }

    async fn get_next(&self) -> Result<Vec<u8>, io::Error >{

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


