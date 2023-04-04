
use std::io;
use tokio::net::UdpSocket;
use tokio;

use crate::core::packet::PacketType;

const BUFFER: usize = 342*8usize;
//Buffer size
pub struct NetListener {
    socket : UdpSocket
}

impl NetListener{
    pub async fn start_listening<T : PacketType>(self){
        println!("Starting listening");
        loop {
            let mut buf = [0;BUFFER];
            let _len = self.socket.recv(&mut buf).await.unwrap();
            let buf = buf.to_vec();
            let packet = T::from_raw_bytes(buf);
            tokio::spawn(async move{
                todo!()
            });
        };
    }

    pub async fn new(address : String) -> io::Result<NetListener>{
        match UdpSocket::bind(address).await{
            Ok(s) => {
                Ok(Self{socket: s})
            },
            Err(e) => Err(e)
        }
    }
}



