pub mod core;
pub mod netio;
use std::time::Duration;

use netio::netlistener::NetListener;
use netio::netoutput::NetSender;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let listener = NetListener::new(String::from("127.0.0.1:3332")).await.unwrap();
    let sender = NetSender::new(String::from("127.0.0.1:3331")).await.unwrap();
    let data = hex_to_bytes("01010600fb2ea1330000000000000000000000000000000000000000b0be8328430e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000063825363350103370c017903060f6c7277fc5f2c2e390205dc3d0701b0be8328430e32048ac33c8333040076a7000c0e4169722d64652d5261706861656cff00").unwrap();
    tokio::spawn(async move {
        let d = data;
        loop {
            sender.send(d.clone(), String::from("127.0.0.1:3332")).await;
            sleep(Duration::from_secs(2)).await;
        }
    });
    listener.start_listening().await;
}

fn hex_to_bytes(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 == 0 {
        (0..s.len())
            .step_by(2)
            .map(|i| s.get(i..i + 2)
                      .and_then(|sub| u8::from_str_radix(sub, 16).ok()))
            .collect()
    } else {
        None
    }
}
