use chrono::{ DateTime, Utc, Duration};
use std::net::Ipv4Addr;
use super::{message_type::DhcpV4Packet, packet_context::{HardwareAddress, PacketContext}};

pub struct LeaseV4 {
     pub ip_address: Ipv4Addr,
     pub expiration : DateTime<Utc>,
     pub hardware_address : HardwareAddress
}

impl LeaseV4 {
    pub fn new(context : PacketContext<DhcpV4Packet, DhcpV4Packet>, duration :Duration) -> Self{
        let expiration_date = Utc::now() + duration;
        let ip = context.output_packet.yiaddr;
        let hardware = context.output_packet.chadd;
        Self{
            ip_address : ip,
            expiration : expiration_date,
            hardware_address : hardware
        }
    }

}