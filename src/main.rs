
use crate::core::packet_context::HardwareAddress;
use ::core::sync;
use std::{sync::{Mutex, Arc}, net::Ipv4Addr, time::Duration};

use chrono::Utc;
use tokio::time::sleep;
use utils::data::{DbManager, RuntimeStorage, DataPool};
use log;

pub mod core;
pub mod netio;
use std::time::Duration;

use netio::netlistener::NetListener;
use netio::netoutput::NetSender;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
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
