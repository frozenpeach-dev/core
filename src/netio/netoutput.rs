use std::io;
use tokio::net::UdpSocket;
use tokio;
use std::sync::Arc;
pub struct NetSender {
    socket : Arc<UdpSocket>
}

impl NetSender {
    // Packet will need to implement pack() that will return raw data to encapsulate in udp datagram
    pub async fn send(&self, data : Vec<u8>, target : String){
        //Sends data to target
        let s = self.socket.clone();
        tokio::spawn(async move {
            s.send_to(&data, target).await.unwrap();
            println!("Sended");
        });
    }

    pub async fn new(address : String) -> io::Result<NetSender>{
        //Creates UdpSocket
        match UdpSocket::bind(address).await{
            Ok(s) => {
                Ok(Self{socket: Arc::new(s)})
            },
            Err(e) => Err(e)
        }
    }

}





