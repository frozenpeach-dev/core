use std::{sync::{Arc, Mutex}, collections::HashMap};

use crate::core::{message_type::DhcpV4Packet, state::PacketState, packet_context::PacketContext, errors::HookError};

use super::{hook_registry::{HookRegistry, Hook}, typemap::TypeMap};




pub fn register_hooks(registry: &mut HookRegistry<DhcpV4Packet, DhcpV4Packet>) {

    let hook_1 = |services: Arc<Mutex<TypeMap>>, packet: &mut PacketContext<DhcpV4Packet, DhcpV4Packet>| {
        let test: &PacketContext<DhcpV4Packet, DhcpV4Packet> = services.try_lock().unwrap().get::<PacketContext<DhcpV4Packet, DhcpV4Packet>>().unwrap();
        println!("test"); Ok::<isize, HookError>(1) };
    registry.register_hook(PacketState::Prepared, Hook::new(String::from("first"), Box::new(hook_1), vec![]));


}
