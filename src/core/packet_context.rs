use std::{time::Duration, net::Ipv4Addr};
use mac_address::MacAddress;

use super::state;

pub struct Packet<T> {
    pub state : state::PacketState,
    pub packet : T
}

pub trait PacketType {

}

impl PacketType for Packet<DhcpPacket>{
}   


pub struct PacketContext<T : PacketType, U> {
    pub input_packet : T,
    pub output_packet : U
}


pub struct DhcpPacket {
    pub htype : u8,
    pub hlen : u8,
    pub hops : u8,
    pub xid : u32,
    pub sec : Duration,
    pub flags : [u8; 2],
    pub ciaddr : Ipv4Addr,
    pub yiaddr : Ipv4Addr,
    pub siaddr : Ipv4Addr,
    pub giaddr : Ipv4Addr,
    pub chadd : HardwareAddress,
    pub sname : String,
    pub file : String,
    pub options : Vec<Option<Vec<u8>>>
}


impl DhcpPacket {
    pub fn from_raw(mut raw : Vec<u8>) -> Self{
        let _mtype = raw.remove(0);
        let htype = raw.remove(0);
        let hlen = raw.remove(0);
        let hops = raw.remove(0);
        let next:[u8; 4] = raw.drain(0..4).as_slice().to_owned().try_into().unwrap();
        let xid = u32::from_le_bytes(next);
        let next: [u8; 2] = raw.drain(0..2).as_slice().to_owned().try_into().unwrap();
        let sec = Duration::from_secs(u16::from_le_bytes(next) as u64);
        let flags = raw.drain(0..2).as_slice().to_owned().try_into().unwrap();
        let a = raw.remove(0);
        let b = raw.remove(0);
        let c = raw.remove(0);
        let d = raw.remove(0);
        let ciaddr = Ipv4Addr::new(a, b, c, d);
        let a = raw.remove(0);
        let b = raw.remove(0);
        let c = raw.remove(0);
        let d = raw.remove(0);
        let yiaddr = Ipv4Addr::new(a, b, c, d);
        let a = raw.remove(0);
        let b = raw.remove(0);
        let c = raw.remove(0);
        let d = raw.remove(0);
        let siaddr = Ipv4Addr::new(a, b, c, d);
        let a = raw.remove(0);
        let b = raw.remove(0);
        let c = raw.remove(0);
        let d = raw.remove(0);
        let giaddr = Ipv4Addr::new(a, b, c, d);
        let next: [u8; 16] = raw.drain(0..16).as_slice().to_owned().try_into().unwrap();
        let chadd = HardwareAddress::new(next);
        let next = raw.drain(0..64).as_slice().to_vec();
        let sname = String::from_utf8_lossy(&next).to_string();
        let next = raw.drain(0..128).as_slice().to_vec();
        let file = String::from_utf8_lossy(&next).to_string();
        let _magic_cookie = raw.drain(0..4).as_slice().to_vec();
        let options = parse_options(raw.to_owned());
        Self { htype: htype, hlen: hlen, hops: hops, xid: xid, sec: sec, flags: flags, ciaddr: ciaddr, yiaddr: yiaddr, siaddr: siaddr, giaddr: giaddr, chadd: chadd, sname: sname, file: file, options: options }

    }
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
    use super::DhcpPacket;
    #[test]
    fn packet_craft(){
        let data = hex_to_bytes("02010601fb2ea0b400000000000000008ac33c830000000000000000b0be8328430e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000638253633501053604c00002013304000062dc0104fffff00003048ac3300106088ac3240a8ac3c2170f1b6f70656e7a6f6e652e63656e7472616c65737570656c65632e6672771f086f70656e7a6f6e650f63656e7472616c65737570656c656302667200c009ff00000000").unwrap();
        let packet = DhcpPacket::from_raw(data);
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