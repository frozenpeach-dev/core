use std::{time::Duration, net::{Ipv4Addr, SocketAddr}};
use chrono::{DateTime, Utc};
use mac_address::MacAddress;

use super::{state::{self, PacketState}, message_type::PacketType};

pub struct PacketContext<T : PacketType, U: PacketType> {

    source_addr : SocketAddr,

    time: DateTime<Utc>,

    id: usize,

    state: PacketState,

    input_packet : T,

    pub output_packet : U

}





pub struct HardwareAddress {
    pub address : MacAddress,
    pub is_mac_address: bool,
    pub raw : [u8; 16]
}

impl HardwareAddress {
    pub fn new(mut raw : [u8; 16]) -> Self{
        let mut i =0;
        raw.reverse();
        while (raw.get(i).unwrap().to_owned() == 0) && (i < 9) {
            i+=1
        }
        raw.reverse();
        let mut addr = MacAddress::new([0; 6]);
        let mut is_mac_addres = false;
        if i == 9 {
            let bytes : [u8;6] = raw[0..6].try_into().unwrap();
            addr = MacAddress::new(bytes);
            is_mac_addres = true;
        }
        Self { address: (addr), is_mac_address: (is_mac_addres), raw: (raw) }
        
    }
}

#[derive(Debug)]
pub struct Option<T> {
    pub code : u8,
    pub value : T
}

pub fn parse_options(mut data : Vec<u8>) -> Vec<Option<Vec<u8>>>{
    let mut result = Vec::new();
    while data.len() > 0 {
        let code = data.remove(0);
        if code == 0u8 {
            continue;
        }
        if code == 255 {
            break;
        }
        let len = data.remove(0) as usize;
        let value = data.drain(0..len).as_slice().to_owned();
        result.push(Option{code, value});
    }
    result
}

#[cfg(test)]


mod packet {
    use crate::core::{packet_context::PacketType, message_type::DhcpV4Packet};

    #[test]
    fn packet_craft(){
        let data = hex_to_bytes("02010601fb2ea0b400000000000000008ac33c830000000000000000b0be8328430e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000638253633501053604c00002013304000062dc0104fffff00003048ac3300106088ac3240a8ac3c2170f1b6f70656e7a6f6e652e63656e7472616c65737570656c65632e6672771f086f70656e7a6f6e650f63656e7472616c65737570656c656302667200c009ff00000000").unwrap();
        let packet = DhcpV4Packet::from_raw_bytes(&data);
        println!("{}", packet.chadd.address);
        println!("{}", packet.yiaddr);
        dbg!("{}", packet.options);
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

}
