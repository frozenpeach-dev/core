use chrono::{ DateTime, Utc, Duration};
use mysql::{params, prelude::{FromRow, FromValue}, Row};
use std::net::Ipv4Addr;
use crate::utils::data::Data;

use super::{message_type::DhcpV4Packet, packet_context::{HardwareAddress, PacketContext}};


pub struct LeaseV4 {
     pub ip_address: Ipv4Addr,
     pub expiration : DateTime<Utc>,
     pub hardware_address : HardwareAddress,
     pub mysql_table : String,
     pub id : u64
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
            mysql_table,
            id : 30
        }
    }
}

impl Data for LeaseV4 {
    fn value(&self) -> mysql::params::Params {
        params! {"id" => self.id, "ip_address" => self.ip_address.to_string(), "hardware_address" => self.hardware_address.to_string(), "expiration" => self.expiration.to_rfc3339()}
    }
    fn insert_statement(&self) -> String {
        format!("INSERT INTO {} VALUES (:id, :ip_address, :hardware_address, :expiration)", self.mysql_table)      
    }
    fn id(&self) -> u64 {
        self.id
    }
}

impl Data for &LeaseV4 {
    fn value(&self) -> mysql::params::Params {
        params! {"id" => self.id, "ip_address" => self.ip_address.to_string(), "hardware_address" => self.hardware_address.to_string(), "expiration" => self.expiration.to_rfc3339()}
    }
    fn insert_statement(&self) -> String {
        format!("INSERT INTO {} VALUES (:id, :ip_address, :hardware_address, :expiration)", self.mysql_table)      
    }
    fn id(&self) -> u64 {
        self.id
    }
}

//Create Lease from mysqlRow
impl FromRow for LeaseV4 {
    fn from_row(row: Row) -> Self
        where
            Self: Sized, {
        todo!()
    }
    fn from_row_opt(row: Row) -> Result<Self, mysql::FromRowError>
        where
            Self: Sized {
        todo!()
    }
}