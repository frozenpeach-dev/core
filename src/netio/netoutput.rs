use std::io;
use tokio::net::UdpSocket;
use tokio;
use std::sync::Arc;
pub struct NetSender {
    socket : Arc<UdpSocket>
}

impl NetSender {
    // A terme la data devra être un packet qui implémentera pack() pour avoir la donnée brute.
    pub async fn send(&self, data : Vec<u8>, target : String){
        let s = self.socket.clone();
        tokio::spawn(async move {
            s.send_to(&data, target).await.unwrap();
            println!("Sended");
        });
    }

    pub async fn new(address : String) -> io::Result<NetSender>{
        match UdpSocket::bind(address).await{
            Ok(s) => {
                Ok(Self{socket: Arc::new(s)})
            },
            Err(e) => Err(e)
        }
    }

}





