use std::{collections::HashMap, sync::{Arc, Mutex}, borrow::BorrowMut};

use crate::core::{message_type::PacketType, packet_context::{self, PacketContext}, state::PacketState, errors::HookError};

use super::typemap::TypeMap;


pub struct HookRegistry<T: PacketType + Send, U: PacketType + Send> {

    registry: HashMap<PacketState, Vec<Box<dyn Fn(Arc<Mutex<TypeMap>>, &mut PacketContext<T, U>) -> Result<(), ()>>>>,
    services: Arc<Mutex<TypeMap>>

}

impl<T: PacketType + Send, U: PacketType + Send> HookRegistry<T, U> {
    pub(crate) async fn run_hooks(&self, packet: &mut packet_context::PacketContext<T, U>) -> Result<(), HookError> {

        for hook in self.registry.get(&packet.state()).unwrap().into_iter() {
            hook(self.services.clone(), packet).unwrap();
        }
        Ok(())

    }

    pub fn register_hook(&mut self, state: PacketState, hook: Box<dyn Fn(Arc<Mutex<TypeMap>>, &mut PacketContext<T, U>) -> Result<(), ()>>) {

        if self.registry.contains_key(&state) {
            self.registry.get_mut(&state).unwrap().push(hook);
        } else {
            self.registry.insert(state, vec![hook]);
        }

    }

    pub fn register_service<V: Sync + Send + 'static>(&mut self, service: V) {
        self.services.try_lock().unwrap().insert(Arc::new(service));
    }
}
