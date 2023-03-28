
use std::io;
use tokio::net::UdpSocket;
use tokio;
pub struct NetListener {
    socket : UdpSocket
}

impl NetListener {
    pub async fn start_listening(self){
        println!("Starting listening");
        loop {
            let mut buf = [0;16384];
            let _len = self.socket.recv(&mut buf).await.unwrap();
            dbg!(buf);
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



