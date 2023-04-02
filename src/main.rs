pub mod core;
pub mod hooks;

use crate::core::{message_type::PacketType, state::PacketState};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

    use crate::core::packet_context::PacketContext;

    use hooks::hook_registry::{HookRegistry, Hook};
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
    struct TestService{
        pub list: Vec<usize>
    }
    impl TestService {
        pub fn add(&mut self, id: usize) { self.list.push(id); }
    }

    fn main() {
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
        hook3.must(hook1.id);
        registry.register_hook(PacketState::Received, hook1);
        let mut packet: PacketContext<A, A> = PacketContext::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 1), 1, input_packet);
        registry.register_hook(PacketState::Received, hook2);
        registry.register_hook(PacketState::Received, hook3);   
        tokio_test::block_on(registry.run_hooks(&mut packet)).unwrap();
    }
