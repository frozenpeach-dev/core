use chrono::{ DateTime, Utc, Duration};
use mysql::params;
use std::net::Ipv4Addr;
use crate::utils::data::Data;

use super::{message_type::DhcpV4Packet, packet_context::{HardwareAddress, PacketContext}};

pub struct LeaseV4 {
     pub ip_address: Ipv4Addr,
     pub expiration : DateTime<Utc>,
     pub hardware_address : HardwareAddress,
     mysql_table : String
}

impl LeaseV4 {
    pub fn new(context : PacketContext<DhcpV4Packet, DhcpV4Packet>, duration :Duration, mysql_table : String) -> Self{
        let expiration_date = Utc::now() + duration;
        let ip = context.output_packet.yiaddr;
        let hardware = context.output_packet.chadd;
        Self{
            ip_address : ip,
            expiration : expiration_date,
            hardware_address : hardware,
            mysql_table
        }
    }
}

impl Data for LeaseV4 {
    fn value(&self) -> mysql::params::Params {
        params! {"ip_address" => self.ip_address.to_string(), "hardware_address" => self.hardware_address.to_string(), "expiration" => self.expiration.to_rfc3339()}
    }
    fn insert_statement(&self) -> String {
        format!("INSERT INTO {} VALUES (?, ?, ?, ?)", self.mysql_table)      
    }
}