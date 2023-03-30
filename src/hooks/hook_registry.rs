use std::{collections::HashMap, sync::{Arc, Mutex}, borrow::BorrowMut, hash::BuildHasherDefault};

use log::{trace, debug};

use crate::core::{message_type::PacketType, packet_context::{self, PacketContext}, state::PacketState, errors::HookError};

use super::{typemap::TypeMap, flags::HookFlags};


pub struct HookRegistry<T: PacketType + Send, U: PacketType + Send> {

    registry: HashMap<PacketState, Vec<Hook<T, U>>>,
    services: Arc<Mutex<TypeMap>>

}

pub struct Hook<T: PacketType + Send, U: PacketType + Send> {
    id: usize,
    name: String,
    dependencies: Vec<usize>,
    flags: Vec<HookFlags>,
    pub exec: Box<dyn Fn(Arc<Mutex<TypeMap>>, &mut PacketContext<T, U>) -> Result<isize, HookError>>,
}

impl<T: PacketType + Send, U: PacketType + Send> Hook<T, U> {

    pub fn new(id: usize, name: String, dependencies: Vec<usize>, hook: Box<dyn Fn(Arc<Mutex<TypeMap>>, &mut PacketContext<T, U>) -> Result<isize, HookError>>, flags: Vec<HookFlags>) -> Self {
        Self {id, name, dependencies, exec: hook, flags}
    }

}

impl<T: PacketType + Send, U: PacketType + Send> HookRegistry<T, U> {

    pub(crate) async fn run_hooks(&self, packet: &mut packet_context::PacketContext<T, U>) -> Result<(), HookError> {

        let mut exec_code: HashMap<usize, isize, nohash_hasher::BuildNoHashHasher<usize>> = HashMap::with_hasher(BuildHasherDefault::default());
    
        for hook in self.registry.get(&packet.state()).clone().unwrap().into_iter() {
            if self._can_execute(&exec_code, &hook.dependencies) {
                (hook.exec)(self.services.clone(), packet)
                     .and_then(|x| {
                         exec_code.insert(hook.id, x);
                         trace!("Hook {} exited successfully (exit code {})", hook.name, x); 
                         Ok(())
                     })
                     .or_else(|x| {
                         if hook.flags.contains(&HookFlags::Fatal) { return Err(x); }
                         else { 
                             exec_code.insert(hook.id, -1);
                             debug!("Hook {} exited with failure (exit code -1)", hook.name);
                             Ok::<(), HookError>(()) 
                         }
                     }).unwrap_err();
            } else {
                debug!("Could not execute hook {} because of previous failures", hook.name);
            }
        }

        Ok(())

    }

    fn _can_execute(&self, exec_code: &HashMap<usize, isize, nohash_hasher::BuildNoHashHasher<usize>>, dependecies: &Vec<usize>) -> bool {

        dependecies.iter().any(|x| {
            exec_code.get(x)
                .and_then(|z| {
                    Some(*z > 0)
                })
                .is_some()
        })

    }


    pub fn register_hook(&mut self, state: PacketState, hook: Hook<T,U> ) {

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
