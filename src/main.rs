use std::{thread, sync::{atomic::{AtomicBool, self, Ordering::SeqCst}, Arc}, time::Duration};

use async_trait::async_trait;
use fp_dhcp::{core::{state_switcher::{Input, StateSwitcher, Output}, packet::{PacketType, PacketContext}, state::PacketState}, hooks::{flags::HookFlag, hook_registry::{HookRegistry, Hook, HookClosure}}};
use tokio::time::{self, Instant, sleep};

pub mod core;
pub mod hooks;
pub mod utils;

#[derive(Clone, Copy)]
    struct A {
        name: usize
    }
    impl PacketType for A {
        fn empty() -> Self {
            Self { name: 1 }
        }
        fn from_raw_bytes(_: &[u8]) -> Self {
            todo!()
        }

        fn to_raw_bytes(&self) -> &[u8] {
            &[0u8; 1]
        }
    }
    impl AsRef<[u8]> for A {
        fn as_ref(&self) -> &[u8] {
            todo!()
        }
    }
    struct SimpleInput {}

    #[async_trait]
    impl Input<A> for SimpleInput {
        async fn get(&self) -> Result<A, std::io::Error> {
            Ok(A::empty())
        }
    }

    struct SimpleOutput {}

    #[async_trait]
    impl Output<A> for SimpleOutput {
        async fn send(&self, packet: A) -> Result<usize, std::io::Error> {
            assert!(packet.name == 5);
            Ok(1)
        } 
    }

    #[tokio::main]
    async fn main() {

        let mut registry: HookRegistry<A, A> = HookRegistry::new();
        registry.register_hook(PacketState::Received, Hook::new(String::from("test_hook"), HookClosure(Box::new(|_, packet: &mut PacketContext<A, A>| {
            packet.get_mut_output().name = 2;
            Ok(1)
        })), vec![HookFlag::Fatal]));
        registry.register_hook(PacketState::Prepared, Hook::new(String::from("test_hook"), HookClosure(Box::new(|_, packet: &mut PacketContext<A, A>| {
            assert!(packet.get_mut_output().name == 2);
            packet.get_mut_output().name = 5;
            Ok(1)
        })), Vec::default()));
        let input = SimpleInput{};
        let output = SimpleOutput{};

        let switch = Arc::new(AtomicBool::new(true));
        let mut state_switcher = StateSwitcher::new(Box::new(input), Box::new(output), registry, switch.clone());

        tokio::spawn(async move {
            sleep(Duration::from_secs(1)).await;
            switch.store(false, SeqCst);
            println!("frost");
        });
        state_switcher.start().await;

        dbg!(state_switcher.drop_count());

    }


