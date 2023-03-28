use enum_iterator::all;

use crate::hooks::hook_registry::HookRegistry;

use super::{message_type::PacketType, packet_context::PacketContext, state::PacketState};



struct PacketForwardingEngine<T: PacketType, U: PacketType>{

    registry: HookRegistry<T, U>,

}

impl<T: PacketType, U: PacketType>PacketForwardingEngine<T, U> {

    pub fn new(registry: HookRegistry<T, U>) -> Self{
        Self{ registry }
    }

    pub async fn run_lifetime(&self, mut packet: PacketContext<T, U>) {

        for state in all::<PacketState>() {

            packet.set_state(state);

            self.registry.run_hooks(&mut packet).unwrap();

        }
    }

}
