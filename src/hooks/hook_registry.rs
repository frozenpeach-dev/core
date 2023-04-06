//! Encapsulated closures to be executed on incoming packets
//! to produce an output using various program-scale services.
//!
//! It provides simple logic for a basic control flow between
//! [`Hook`].
//!
//! This module defines [`Hook`] that encapsulates the closures, 
//! and a [`HookRegistry`] to store [`Hook`] and services.


use std::{collections::{HashMap, hash_map::Entry}, sync::{Arc, Mutex}};

use itertools::Itertools;
use log::{trace, debug};
use uuid::Uuid;


use crate::core::{state::PacketState, errors::HookError, packet::{PacketType, PacketContext}};

use super::{typemap::TypeMap, flags::HookFlag};

pub struct HookClosure<T: PacketType, U: PacketType>(pub Box<dyn Fn(Arc<Mutex<TypeMap>>, &mut PacketContext<T, U>) -> Result<isize, HookError>>);
unsafe impl<T: PacketType, U: PacketType> Send for HookClosure<T, U>{}
unsafe impl<T: PacketType, U: PacketType> Sync for HookClosure<T, U>{}

/// An encapsulated closure, to be executed on a [`PacketContext`]
/// to perform all types of actions. They make most of the
/// actual logic of the program.
///
/// They can be created from a given name and a given closure
///
/// Names are only there for identification purposes for now.
///
/// They are uniquely identified all around the program
/// using a [`Uuid`] generated at creation time.
/// They also implement a simple logic to link together.
/// You can execute a `Hook` based on the success or not
/// of another `Hook`
///
/// A `Hook` can also hold one or more [`HookFlag`] to control
/// its execution flow.
pub struct Hook<T: PacketType + Send, U: PacketType + Send> {
    id: Uuid,
    name: String,
    dependencies: HashMap<Uuid, bool>,
    flags: Vec<HookFlag>,
    exec: HookClosure<T, U> 
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
    pub fn new(name: String, exec: HookClosure<T, U>, flags: Vec<HookFlag>) -> Self {
        let id = Uuid::new_v4();
        Self {id, name, dependencies: HashMap::new(), exec, flags} 
    }

    /// Retrieve the [`Uuid`] belonging to a [`Hook`] 
    ///
    /// # Examples:
    ///
    /// ```
    /// let test_hook = Hook::new("My hook", Box::new(|_, _| {} ));
    /// println!(test_hook.id());
    /// ```

    pub fn id(&self) -> Uuid{
        self.id
    }

    /// Add a new [`HookFlag`] to this `Hook`
    ///
    /// # Examples:
    ///
    /// ```
    /// let test_hook = Hook::new("My hook", Box::new(|_, _| {} ));
    /// test_hook.add_flag(HookFlags::Fatal);
    /// ```

    pub fn add_flag(&mut self, new_flag: HookFlag) {
        self.flags.push(new_flag);
    }

    /// Retrieve the different [`HookFlag`] associated 
    /// to this `Hook`
    /// # Examples:
    ///
    /// ```
    /// let test_hook = Hook::new("My hook", Box::new(|_, _| {} ));
    /// test_hook.add_flag(HookFlag::Fatal);
    /// test_hook.flags().contains(&HookFlag::Fatal);
    /// ```

    pub fn flags(&self) -> &Vec<HookFlag>{
        &self.flags
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

/// A register to store and manage the different [`Hook`]
/// to be executed on the packets. It also stores various services 
/// instances which can then be called by the [`Hook`] to perform
/// logic at the program scale. 
pub struct HookRegistry<T: PacketType + Send, U: PacketType + Send> {

    registry: HashMap<PacketState, HashMap<Uuid, Hook<T, U>>>,
    services: Arc<Mutex<TypeMap>>,
    exec_order: HashMap<PacketState, Vec<Uuid>>,
    need_update: bool

}

impl<T: PacketType + Send, U: PacketType + Send> Default for HookRegistry<T, U> {

    fn default() -> Self {
        Self::new()
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
        Self { registry: HashMap::new(), services: Arc::new(Mutex::new(TypeMap::new())), exec_order: HashMap::new(), need_update: true}
    }

    /// Execute every registered [`Hook`] on the given [`PacketContext`] 
    /// for its current state
    ///
    /// # Errors
    ///
    /// Returns [`HookError`] if any [`Hook`] holding the [`Fatal`]
    /// flag panics.
    ///
    /// [`Fatal`]: crate::hooks::flags::HookFlag::Fatal
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
    pub fn run_hooks(&self, packet: &mut PacketContext<T, U>) -> Result<(), HookError> {
    
        if self.need_update {
            return Err(HookError::new("Circular dependencies in hooks"));
        }

        let mut exec_code: HashMap<Uuid, isize> = HashMap::new();        
        if packet.state() == PacketState::Failure {
            self.run_failure_chain(packet)?
        }

        let exec_order = match self.exec_order.get(&packet.state()) {
            Some(order) => order,
            None => { return Ok(()); }
        };

        for hook in exec_order.iter() {

            let hook = match self.registry.get(&packet.state()) {
                Some(lst) => { match lst.get(hook) {
                    Some(hook) => hook,
                    None => { continue; }
                }},
                None => { continue; }
            };

            if exec_code.contains_key(&hook.id) { continue; }

            if self.can_execute(&exec_code, &hook.dependencies) {
                (hook.exec.0)(self.services.clone(), packet)
                    .map(|x| {
                        exec_code.insert(hook.id, x);
                        trace!("Hook {} exited successfully (exit code {})", hook.name, x); 
                    })
                    .or_else(|_| {
                        if hook.flags.contains(&HookFlag::Fatal) { self.run_failure_chain(packet) }
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

        self.need_update = true;
        if let Entry::Vacant(e) = self.registry.entry(state) {
            e.insert(HashMap::new());
            self.register_hook(state, hook);
        } else {
            self.registry.get_mut(&state).unwrap().insert(hook.id, hook);
        }
        if let Ok(order) = self.generate_exec_order(&state) {
            self.exec_order.insert(state, order);
            self.need_update = false;
        }

    }

    /// Insert a new service inside the [`HookRegistry`]
    ///
    /// The service's type must implement the following traits:
    /// [`Send`] and [`Sync`]

    pub fn register_service<V: Send + Sync + 'static>(&mut self, service: V) {
        self.services.lock().expect("Services mutex was poisonned")
            .insert(Arc::new(service));
    }

    fn run_failure_chain(&self, packet: &mut PacketContext<T, U>) -> Result<(), HookError> {
        
        for hook in self.registry.get(&PacketState::Failure).ok_or(HookError::new("No failure hooks defined"))?.values() {
            (hook.exec.0)(self.services.clone(), packet)
                .or_else(|x| {
                    debug!("Hook {} in failure chain exited with failure (exit code {})", hook.name, x);
                    Ok::<isize, HookError>(0)
                }).unwrap();
        }
        Err(HookError::new("One or more fatal hooks was unsuccessful"))

    }

    fn can_execute(&self, exec_code: &HashMap<Uuid, isize>, dependencies: &HashMap<Uuid, bool>) -> bool {

        !dependencies.iter().any(|(x, need_success)| {
            exec_code.get(x)
                .map(|z| (*z < 0 && *need_success) | (*z >=0 && !*need_success))
                .unwrap_or_default()
        })

    }

    fn generate_exec_order(&self, for_state: &PacketState) -> Result<Vec<Uuid>, HookError>{
        
        let mut deps_map : HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut resolved_graph : Vec<Uuid> = Vec::new();

        for hook in self.registry.get(for_state).ok_or(HookError::new("No hooks associated with this state"))?.iter() {
            deps_map.insert(*hook.0, hook.1.dependencies.keys().copied().collect_vec());
        }

        while !deps_map.is_empty() {

            let mut ready_hooks : Vec<Uuid> = Vec::new();
            
            for (hook, deps) in deps_map.iter() {
                if deps.is_empty() {
                    ready_hooks.push(*hook);
                }
            } 

            if ready_hooks.is_empty() {
                return Err(HookError::new("Circular dependencies in hooks"));
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

    use super::*;
    #[derive(Clone, Copy)]
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

        fn to_raw_bytes(&self) -> &[u8] {
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
        registry.register_hook(PacketState::Received, Hook::new(String::from("test_hook"), HookClosure(Box::new(|_, packet: &mut PacketContext<A, A>| {
            packet.get_mut_output().name = 2;
            Ok(1)
        })), Vec::default()));

        let mut packet: PacketContext<A, A> = PacketContext::from(input_packet);

        assert!(packet.get_output().name == 0);
        registry.run_hooks(&mut packet).unwrap();
        assert!(packet.get_output().name == 2);

    }

    #[test]
    fn test_dependency_hook() {
        let mut registry: HookRegistry<A, A> = HookRegistry::new();
        let input_packet = A::empty(); 
        let hook1 = Hook::new(String::from("test1"), HookClosure(Box::new(|_, _| {
            Ok(1)
        })), Vec::default());
        let hook2 = Hook::new(String::from("test2"), HookClosure(Box::new(|_, _| {
            Ok(1)
        })), Vec::default());
        let mut hook3 = Hook::new(String::from("test2"), HookClosure(Box::new(|_, _| {
            assert!(0 == 1); 
            Ok(1)
        })), Vec::default());
        hook3.must_not(hook1.id);
        registry.register_hook(PacketState::Received, hook1);
        let mut packet: PacketContext<A, A> = PacketContext::from(input_packet);
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
        registry.register_hook(PacketState::Received, Hook::new(String::from("test_hook"), HookClosure(Box::new(|serv, packet: &mut PacketContext<A, A>| {
            let mut serv_mgr = serv.try_lock().unwrap();
            let my_serv = serv_mgr.get_mut::<Arc<Mutex<TestService>>>().unwrap();
            my_serv.try_lock().unwrap().add(packet.get_output().name);
            my_serv.try_lock().unwrap().add(packet.get_output().name);
            packet.get_mut_output().name = 2;
            Ok(1)
        })), Vec::default()));

        let mut packet: PacketContext<A, A> = PacketContext::from(input_packet);

        registry.run_hooks(&mut packet).unwrap();
        assert!(registry.services.try_lock().unwrap().get::<Arc<Mutex<TestService>>>().unwrap().try_lock().unwrap().list.len() == 2);

    }

    #[test]
    fn test_dependency_tree() {
        let mut registry: HookRegistry<A, A> = HookRegistry::new();

        let mut hook1 = Hook::new(String::from("test1"), HookClosure(Box::new(|_, _: &mut PacketContext<A, A>| {
            Ok(1)
        })), Vec::default());
        let mut hook2 = Hook::new(String::from("test2"), HookClosure(Box::new(|_, _: &mut PacketContext<A, A>| {
            Ok(1)
        })), Vec::default());
        let hook3 = Hook::new(String::from("test2"), HookClosure(Box::new(|_, _: &mut PacketContext<A, A>| {
            assert!(0 == 1); 
            Ok(1)
        })), Vec::default());

        let hook1id = hook1.id;
        let hook2id = hook2.id;
        let hook3id = hook3.id;

        hook2.must(hook1id);
        hook1.must(hook3id);
        hook2.must(hook3id);

        registry.register_hook(PacketState::Received, hook3);
        registry.register_hook(PacketState::Received, hook2);
        registry.register_hook(PacketState::Received, hook1);

        let mut graph = registry.generate_exec_order(&PacketState::Received).unwrap();

        assert!(graph.pop().unwrap() == hook2id);
        assert!(graph.pop().unwrap() == hook1id);
        assert!(graph.pop().unwrap() == hook3id);
    }

}

