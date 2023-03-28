use std::collections::HashMap;

use crate::core::{message_type::PacketType, packet_context::{self, PacketContext}, state::PacketState};



pub struct HookRegistry<T: PacketType, U: PacketType> {

    registry: HashMap<PacketState, Vec<Box<dyn Fn(&mut PacketContext<T, U>) -> Result<(), ()>>>>

}
impl<T: PacketType, U: PacketType> HookRegistry<T, U> {
    pub(crate) async fn run_hooks(&self, packet: &mut packet_context::PacketContext<T, U>) -> Result<(), ()> {

        for hook in self.registry.get(&packet.state()).unwrap().into_iter() {
    
            hook(packet).unwrap();

        }
        Ok(())

    }
}
