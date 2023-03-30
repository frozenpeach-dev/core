use enum_iterator::all;

use crate::hooks::hook_registry::HookRegistry;

use super::{message_type::PacketType, packet_context::PacketContext, state::PacketState, errors::HookError};



struct PacketForwardingEngine<T: PacketType + Send, U: PacketType + Send>{

    registry: HookRegistry<T, U>,

}

impl<T: PacketType + Send, U: PacketType + Send>PacketForwardingEngine<T, U> {

    pub fn new(registry: HookRegistry<T, U>) -> Self{
        Self{ registry }
    }

    pub async fn run_lifetime(&self, mut packet: PacketContext<T, U>) -> Result<(), HookError>{

        for state in all::<PacketState>() {

            packet.set_state(state);

            self.registry.run_hooks(&mut packet).await?

        }

        Ok(())
    }

}
