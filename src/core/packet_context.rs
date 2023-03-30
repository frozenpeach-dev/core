use std::{time::Duration, net::{Ipv4Addr, SocketAddr}};
use chrono::{DateTime, Utc, NaiveTime};
use enum_iterator::Sequence;
use mac_address::MacAddress;


use super::{state::{self, PacketState}, message_type::PacketType};

pub struct PacketContext<T : PacketType, U: PacketType> {
    pub source_addr : SocketAddr,
    time: DateTime<Utc>,
    id: usize,
    state: PacketState,
    pub input_packet : T,
    pub output_packet : U

}

impl<T: PacketType, U:PacketType> PacketContext<T, U> {

    pub fn set_state(&mut self, new_state: PacketState) {
        self.state = new_state;
    }

}


// sussiest code ever written 
impl<T: PacketType, U:PacketType> Iterator for PacketContext<T, U> {
    type Item = Result<(), ()>;

    fn next(&mut self) -> std::option::Option<Self::Item> {

        self.set_state(self.state.next().unwrap()); 

        Some(Ok(()))

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

