use crate::core::{packet::PacketType, state_switcher::Input};



pub struct UdpInput {

}

impl<U: PacketType + Sync> Input<U> for UdpInput {
    fn get(&self) -> U {
        todo!()
    }
}
