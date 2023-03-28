use enum_iterator::all;

use crate::hooks::hook_registry::HookRegistry;

use super::{message_type::PacketType, packet_context::PacketContext, state::PacketState};



struct PacketForwardingEngine {

    registry: HookRegistry,

}

impl PacketForwardingEngine {

    pub fn new(registry: HookRegistry) -> Self{
        Self{ registry }
    }

    pub async fn run_lifetime<T: PacketType, U: PacketType>(&self, mut packet: PacketContext<T, U>) {

        for state in all::<PacketState>() {

            packet.set_state(state);

            self.registry.run_hooks(&mut packet).unwrap();

        }
    }

}
