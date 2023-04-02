use std::{collections::HashMap, sync::{Arc, Mutex}};

use itertools::Itertools;
use log::{trace, debug};
use uuid::Uuid;


use crate::core::{message_type::PacketType, packet_context::{self, PacketContext}, state::PacketState, errors::HookError};

use super::{typemap::TypeMap, flags::HookFlags};

pub struct HookRegistry<T: PacketType + Send, U: PacketType + Send> {

    registry: HashMap<PacketState, HashMap<Uuid, Hook<T, U>>>,
    services: Arc<Mutex<TypeMap>>

}

pub struct Hook<T: PacketType + Send, U: PacketType + Send> {
    pub id: Uuid,
    name: String,
    pub dependencies: HashMap<Uuid, bool>,
    flags: Vec<HookFlags>,
    next_hook: Option<Uuid>,
    pub exec: Box<dyn Fn(Arc<Mutex<TypeMap>>, &mut PacketContext<T, U>) -> Result<isize, HookError>>,
}

impl<T: PacketType + Send, U: PacketType + Send> Hook<T, U> {

    /// Creates a new `Hook` using the specified closure
    ///
    /// A random [`Uuid`] is generated to represent the `Hook`
    ///
    /// The closure must be in a [`Box`], and must expect two arguments:
    ///
    /// [`TypeMap`] enclosed in an [`Arc`] and [`Mutex`]
    /// and a mutable reference to a [`PacketContext`]
    ///
    /// It must return a [`Result`], which can be either a [`i32`] number, positive in case of
    /// success, negative otherwise, or a [`HookError`]
    /// # Examples:
    ///
    /// ```
    /// let my_hook = Hook::new("My hook", Box::new(|services, packet| { println!(packet.id); }));
    /// ```
    pub fn new(name: String, exec: Box<dyn Fn(Arc<Mutex<TypeMap>>, &mut PacketContext<T, U>) -> Result<isize, HookError>>, flags: Vec<HookFlags>) -> Self {
        let id = Uuid::new_v4();
        Self {id, name, dependencies: HashMap::new(), exec, flags, next_hook: None}
    }

    /// Add a dependency to the success of another `Hook` specified by its [`Uuid`]
    ///
    /// # Examples:
    ///
    /// ```
    /// let my_hook = Hook::new("Hook1", Box::new(|_, _| { }));
    /// let dependent_hook = Hook::new("Hook2", Box::new(|_, _| {} ));
    ///
    /// dependent_hook.must(my_hook.id);
    /// ```

    pub fn must(&mut self, hook: Uuid) {
        self.dependencies.insert(hook, true);
    }

    /// Add a dependency to the failure of another `Hook` specified by its [`Uuid`]
    ///
    /// # Examples:
    ///
    /// ```
    /// let my_hook = Hook::new("Hook1", Box::new(|_, _| { }));
    /// let dependent_hook = Hook::new("Hook2", Box::new(|_, _| {} ));
    ///
    /// dependent_hook.must(my_hook.id);
    /// ```
    pub fn must_not(&mut self, hook: Uuid) {
        self.dependencies.insert(hook, false);
    }

}

impl<T: PacketType + Send, U: PacketType + Send> HookRegistry<T, U> {

    /// Creates a new `HookRegistry`
    ///
    /// This does not allocate initial buffers for 
    /// the underlying registries for [`Hook`] or services 
    ///
    /// # Examples
    ///
    /// ```
    /// let registry = HookRegistry::new();
    /// ```
    pub fn new() -> Self {
        Self { registry: HashMap::new(), services: Arc::new(Mutex::new(TypeMap::new()))}
    }

    /// Execute every registered [`Hook`] on the given [`PacketContext`] 
    /// for its current state
    ///
    /// # Errors
    ///
    /// Returns [`HookError`] if any [`Hook`] holding the [`Fatal`]
    /// flag panics.
    ///
    /// [`Fatal`]: crate::hooks::flags::HookFlags::Fatal
    ///
    /// # Examples
    ///
    /// ```
    /// let mut registry = HookRegistry::new();
    /// let my_hook = Hook::new("My hook", Box::new(|services, packet| { println!(packet.id); }));
    /// registry.register_hook(PacketState::Received, my_hook);
    /// let mut packet: PacketContext<A, A> = PacketContext::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 1), 1, input_packet);
    /// 
    /// registry.run_hooks(packet);
    /// ```
    ///
    /// This will print out a 1
    pub fn run_hooks(&self, packet: &mut packet_context::PacketContext<T, U>) -> Result<(), HookError> {
    
        let mut exec_code: HashMap<Uuid, isize> = HashMap::new();        
        if packet.state() == &PacketState::Failure {
            self.run_failure_chain(packet)?
        }

        let exec_order = self.generate_graph(&packet.state())?;

        for hook in exec_order.iter() {

            let hook = self.registry.get(&packet.state()).unwrap().get(hook).unwrap();

            if exec_code.contains_key(&hook.id) { continue; }

            if self.can_execute(&exec_code, &hook.dependencies) {
                (hook.exec)(self.services.clone(), packet)
                    .and_then(|x| {
                        exec_code.insert(hook.id, x);
                        trace!("Hook {} exited successfully (exit code {})", hook.name, x); 
                        Ok(())
                    })
                    .or_else(|_| {
                        if hook.flags.contains(&HookFlags::Fatal) { return self.run_failure_chain(packet); }
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

    /// Insert a new [`Hook`] inside the [`HookRegistry`] 
    /// for a given [`PacketState`]
    ///
    /// # Examples
    ///
    /// ```
    /// let mut registry = HookRegistry::new();
    /// let my_hook = Hook::new("My hook", Box::new(|_, _| { }));
    /// registry.register_hook(PacketState::Received, my_hook);
    /// ```

    pub fn register_hook(&mut self, state: PacketState, hook: Hook<T,U> ) {

        if self.registry.contains_key(&state) {
            self.registry.get_mut(&state).unwrap().insert(hook.id, hook);

        } else {
            self.registry.insert(state, HashMap::new());
            self.register_hook(state, hook);
        }

    }

    /// Insert a new service inside the [`HookRegistry`]
    ///
    /// The service's type must implement the following traits:
    /// [`Send`] and [`Sync`]

    pub fn register_service<V: Send + Sync + 'static>(&mut self, service: V) {
        self.services.try_lock().unwrap().insert(Arc::new(service));
    }

    fn run_failure_chain(&self, packet: &mut PacketContext<T, U>) -> Result<(), HookError> {
        
        for hook in self.registry.get(&PacketState::Failure).unwrap().values() {
            (hook.exec)(self.services.clone(), packet)
                .or_else(|x| {
                    debug!("Hook {} in failure chain exited with failure (exit code {})", hook.name, x);
                    Ok::<isize, HookError>(0)
                }).unwrap();
        }
        Err(HookError::new(0))

    }

    fn can_execute(&self, exec_code: &HashMap<Uuid, isize>, dependencies: &HashMap<Uuid, bool>) -> bool {

        !dependencies.iter().any(|(x, need_success)| {
            exec_code.get(x)
                .and_then(|z| {
                    Some((*z < 0 && *need_success) | (*z >= 0 && !*need_success))
                })
                .unwrap_or_default()
        })

    }

    fn generate_graph(&self, for_state: &PacketState) -> Result<Vec<Uuid>, HookError>{
        
        let mut deps_map : HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut resolved_graph : Vec<Uuid> = Vec::new();

        for hook in self.registry.get(for_state).unwrap().iter() {
            deps_map.insert(*hook.0, hook.1.dependencies.keys().map(|x| *x).collect_vec());
        }

        while deps_map.len() > 0 {

            let mut ready_hooks : Vec<Uuid> = Vec::new();
            
            for (hook, deps) in deps_map.iter() {
                if deps.len() == 0 {
                    ready_hooks.push(*hook);
                }
            } 

            if ready_hooks.len() == 0 {
                return Err(HookError::new(4));
            }

            for hook in ready_hooks.iter() {
                deps_map.remove(hook);
                resolved_graph.push(*hook);
            }

            for (_, deps) in deps_map.iter_mut() {
                deps.retain(|x| !ready_hooks.contains(x));
            }

        } 
        Ok(resolved_graph)
    }

}

#[cfg(test)]
mod tests {

    use std::net::{SocketAddr, IpAddr, Ipv4Addr};

    use super::*;
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
    struct TestService{
        pub list: Vec<usize>
    }
    impl TestService {
        pub fn add(&mut self, id: usize) { self.list.push(id); }
    }
    #[test]
    fn test_simple_hook() {

        let mut registry: HookRegistry<A, A> = HookRegistry::new();
        let input_packet = A::empty();
        registry.register_hook(PacketState::Received, Hook::new(String::from("test_hook"), Box::new(|_, packet: &mut PacketContext<A, A>| {
            packet.output_packet.name = 2;
            Ok(1)
        }), Vec::default()));

        let mut packet: PacketContext<A, A> = PacketContext::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 1), 1, input_packet);

        assert!(packet.output_packet.name == 0);
        registry.run_hooks(&mut packet).unwrap();
        assert!(packet.output_packet.name == 2);

    }

    #[test]
    fn test_dependency_hook() {
        let mut registry: HookRegistry<A, A> = HookRegistry::new();
        let input_packet = A::empty(); 
        let mut hook1 = Hook::new(String::from("test1"), Box::new(|_, _| {
            Ok(1)
        }), Vec::default());
        let mut hook2 = Hook::new(String::from("test2"), Box::new(|_, _| {
            Ok(1)
        }), Vec::default());
        let mut hook3 = Hook::new(String::from("test2"), Box::new(|_, _| {
            assert!(0 == 1); 
            Ok(1)
        }), Vec::default());
        hook3.must_not(hook1.id);
        registry.register_hook(PacketState::Received, hook1);
        let mut packet: PacketContext<A, A> = PacketContext::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 1), 1, input_packet);
        registry.register_hook(PacketState::Received, hook2);
        registry.register_hook(PacketState::Received, hook3);   
        registry.run_hooks(&mut packet).unwrap();
    }

    #[test]
    fn test_service() {
        let test_service: TestService = TestService { list: Vec::new() };

        let mut registry: HookRegistry<A, A> = HookRegistry::new();
        registry.register_service(Mutex::new(test_service));
        let input_packet = A::empty();
        registry.register_hook(PacketState::Received, Hook::new(String::from("test_hook"), Box::new(|serv, packet: &mut PacketContext<A, A>| {
            let mut serv_mgr = serv.try_lock().unwrap();
            let my_serv = serv_mgr.get_mut::<Arc<Mutex<TestService>>>().unwrap();
            my_serv.try_lock().unwrap().add(packet.output_packet.name);
            my_serv.try_lock().unwrap().add(packet.output_packet.name);
            packet.output_packet.name = 2;
            Ok(1)
        }), Vec::default()));

        let mut packet: PacketContext<A, A> = PacketContext::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 1), 1, input_packet);

        registry.run_hooks(&mut packet).unwrap();
        assert!(registry.services.try_lock().unwrap().get::<Arc<Mutex<TestService>>>().unwrap().try_lock().unwrap().list.len() == 2);

    }

    #[test]
    fn test_dependency_tree() {
        let mut registry: HookRegistry<A, A> = HookRegistry::new();
    }

}
