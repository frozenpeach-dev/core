use crate::core::{message_type::PacketType, packet_context};



pub struct HookRegistry {

    

}
impl HookRegistry {
    pub(crate) fn run_hooks<T: PacketType, U: PacketType>(&self, packet: &mut packet_context::PacketContext<T, U>) -> Result<(), ()> {
        todo!()
    }
}
