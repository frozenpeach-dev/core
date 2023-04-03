use chrono::{ DateTime, Utc, Duration};
use mysql::{params, prelude::FromRow, Row};
use std::{net::Ipv4Addr, str::FromStr};
use crate::utils::data::Data;

use super::{message_type::DhcpV4Packet, packet_context::{HardwareAddress, PacketContext}};


pub struct LeaseV4 {
     pub ip_address: Ipv4Addr,
     pub expiration : DateTime<Utc>,
     pub hardware_address : HardwareAddress,
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
            id : 30
        }
    }
}

impl Data for LeaseV4 {
    fn value(&self) -> mysql::params::Params {
        params! {"id" => self.id, "ip_address" => self.ip_address.to_string(), "hardware_address" => self.hardware_address.to_string(), "expiration" => self.expiration.to_rfc3339()}
    }
    fn insert_statement(&self, place : String) -> String {
        format!("INSERT INTO {} VALUES (:id, :ip_address, :hardware_address, :expiration)", place)     
    }
    fn id(&self) -> u64 {
        self.id
    }
}

impl Data for &LeaseV4 {
    fn value(&self) -> mysql::params::Params {
        params! {"id" => self.id, "ip_address" => self.ip_address.to_string(), "hardware_address" => self.hardware_address.to_string(), "expiration" => self.expiration.to_rfc3339()}
    }
    fn insert_statement(&self, place : String) -> String {
        format!("INSERT INTO {} VALUES (:id, :ip_address, :hardware_address, :expiration)", place)     
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
                let id :u64= row.get(0).unwrap();
                let ip : String = row.get(1).unwrap();
                let ip = Ipv4Addr::from_str(&ip).unwrap();
                let expiration : String = row.get(2).unwrap();
                let expiration:DateTime<Utc> = DateTime::from_str(&expiration).unwrap();
                let hardware: String = row.get(3).unwrap();
                let hardware = HardwareAddress::new([0; 16]);
                Self { ip_address: ip, expiration, hardware_address: hardware, id}

    }
    fn from_row_opt(_row: Row) -> Result<Self, mysql::FromRowError>
        where
            Self: Sized {
        todo!()
    }
}