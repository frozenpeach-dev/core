use std::sync::{Arc, Mutex};

use crate::core::{message_type::DhcpV4Packet, state::PacketState, packet_context::PacketContext};

use super::{hook_registry::HookRegistry, typemap::TypeMap};




pub fn register_hooks(registry: &mut HookRegistry<DhcpV4Packet, DhcpV4Packet>) {

    let hook_1 = |services: Arc<Mutex<TypeMap>>, packet: &mut PacketContext<DhcpV4Packet, DhcpV4Packet>| {
        let test: &PacketContext<DhcpV4Packet, DhcpV4Packet> = services.try_lock().unwrap().get::<PacketContext<DhcpV4Packet, DhcpV4Packet>>().unwrap();
        println!("test"); Ok::<(), ()>(()) };
    registry.register_hook(PacketState::Prepared, Box::new(hook_1));

}
