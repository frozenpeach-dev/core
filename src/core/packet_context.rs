use std::{time::Duration, net::{IpAddr, Ipv4Addr}, vec, io::Read};

use mac_address::MacAddress;

pub trait Packet {
}
pub struct PacketContext<T : Packet, U> {
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
        let sname = String::from_utf8(next).unwrap();
        let next = raw.drain(0..128).as_slice().to_vec();
        let file = String::from_utf8(next).unwrap();
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
    pub fn new(raw : [u8; 16]) -> Self{
        let mut i =0;
        while (*raw.get(i).unwrap() != 0u8) & (i < 10) {
            i+=1
        }
        let mut addr = MacAddress::new([0; 6]);
        let mut is_mac_addres = false;
        if i == 9 {
            let bytes : [u8;6] = raw[9..15].try_into().unwrap();
            addr = MacAddress::new(bytes);
            is_mac_addres = true;
        }
        Self { address: (addr), is_mac_address: (is_mac_addres), raw: (raw) }
        
    }
}
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
