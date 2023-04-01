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
    dependencies: HashMap<usize, bool>,
    flags: Vec<HookFlags>,
    pub exec: Box<dyn Fn(Arc<Mutex<TypeMap>>, &mut PacketContext<T, U>) -> Result<isize, HookError>>,
}

impl<T: PacketType + Send, U: PacketType + Send> Hook<T, U> {

    pub fn new(id: usize, name: String, dependencies: HashMap<usize, bool>, hook: Box<dyn Fn(Arc<Mutex<TypeMap>>, &mut PacketContext<T, U>) -> Result<isize, HookError>>, flags: Vec<HookFlags>) -> Self {
        Self {id, name, dependencies, exec: hook, flags}
    }

}

impl<T: PacketType + Send, U: PacketType + Send> HookRegistry<T, U> {

    pub fn new() -> Self {
        Self { registry: HashMap::new(), services: Arc::new(Mutex::new(TypeMap::new()))}
    }

    pub(crate) async fn run_hooks(&self, packet: &mut packet_context::PacketContext<T, U>) -> Result<(), HookError> {
    
        let mut exec_code: HashMap<usize, isize, nohash_hasher::BuildNoHashHasher<usize>> = HashMap::with_hasher(BuildHasherDefault::default());
        
        if packet.state() == &PacketState::Failure {
            self._run_failure_chain(packet)?
        }

        for hook in self.registry.get(&packet.state()).clone().unwrap().into_iter() {
            if self._can_execute(&exec_code, &hook.dependencies) {
                (hook.exec)(self.services.clone(), packet)
                     .and_then(|x| {
                         exec_code.insert(hook.id, x);
                         trace!("Hook {} exited successfully (exit code {})", hook.name, x); 
                         Ok(())
                     })
                     .or_else(|_| {
                         if hook.flags.contains(&HookFlags::Fatal) { return self._run_failure_chain(packet); }
                         else { 
                             exec_code.insert(hook.id, -1);
                             debug!("Hook {} exited with failure (exit code -1)", hook.name);
                             Ok::<(), HookError>(()) 
                         }
                     }).unwrap();
            } else {
                trace!("Skipped execution of hook {} because of unmet requirements", hook.name);
            }
        }

        Ok(())

    }

    fn _run_failure_chain(&self, packet: &mut PacketContext<T, U>) -> Result<(), HookError> {
        
        Ok(()) 

    }

    fn _can_execute(&self, exec_code: &HashMap<usize, isize, nohash_hasher::BuildNoHashHasher<usize>>, dependencies: &HashMap<usize, bool>) -> bool {

        !dependencies.iter().any(|(x, need_success)| {
            exec_code.get(x)
                .and_then(|z| {
                    Some((*z < 0 && *need_success) | (*z >= 0 && !*need_success))
                })
                .unwrap_or_default()
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

#[cfg(test)]
mod tests {

    use std::net::{SocketAddr, IpAddr, Ipv4Addr};

    use super::*;
    use tokio_test;
    struct A {
        name: usize
    }
    impl PacketType for A {
        fn empty() -> Self {
            Self { name: 0 }
        }
        fn from_raw_bytes(_: &[u8]) -> Self {
            todo!()
        }
    }
    impl AsRef<[u8]> for A {
        fn as_ref(&self) -> &[u8] {
            todo!()
        }
    }
    #[test]
    fn test_simple_hook() {

        let mut registry: HookRegistry<A, A> = HookRegistry::new();
        let input_packet = A::empty();
        registry.register_hook(PacketState::Received, Hook::new(1, String::from("test_hook"), HashMap::default(), Box::new(|_, packet| {
            packet.output_packet.name = 2;
            Ok(1)
        }), Vec::default()));

        let mut packet: PacketContext<A, A> = PacketContext::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 1), 1, input_packet);

        assert!(packet.output_packet.name == 0);
        tokio_test::block_on(registry.run_hooks(&mut packet)).unwrap();
        assert!(packet.output_packet.name == 2);

    }

    #[test]
    fn test_dependency_hook() {
        let mut registry: HookRegistry<A, A> = HookRegistry::new();
        let input_packet = A::empty();
        registry.register_hook(PacketState::Received, Hook::new(1, String::from("test1"), HashMap::default(), Box::new(|_, packet| {
            packet.output_packet.name = 3;
            Ok(1)
        }), Vec::default()));
        let mut deps = HashMap::new();
        let mut packet: PacketContext<A, A> = PacketContext::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 1), 1, input_packet);
        deps.insert(1, true);
        registry.register_hook(PacketState::Received, Hook::new(2, String::from("test2"), deps, Box::new(|_, packet| {
            assert!(packet.output_packet.name == 3);
            packet.output_packet.name = 4;
            assert!(packet.output_packet.name == 4);
            
            Ok(1)
        }), Vec::default()));
        let mut deps2 = HashMap::new();
        deps2.insert(1, true);
        deps2.insert(2, false);
        registry.register_hook(PacketState::Received, Hook::new(3, String::from("test2"), deps2, Box::new(|_, packet| {
            assert!(0 == 1); 
            Ok(1)
        }), Vec::default()));   
        tokio_test::block_on(registry.run_hooks(&mut packet)).unwrap();
    }

}
