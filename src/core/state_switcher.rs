//! State switching logic implementation for [`PacketContext`]
//! A `StateSwitcher` runs all hooks defined for a given
//! state before transitioning to the next state
//!
//! `Input` and `Output` are abstract types
//! used to gather incoming data and dispatch
//! outgoing one.

use std::sync::{Arc, atomic::{AtomicUsize, Ordering::SeqCst, AtomicBool}};

use async_trait::async_trait;
use crate::hooks::hook_registry::HookRegistry;

use super::{packet::{PacketType, PacketContext}, state::PacketState};

#[async_trait]
pub trait Output<T: PacketType>: Send + Sync {
    async fn send(&self, packet: T) -> Result<usize, std::io::Error>;
}

#[async_trait]
pub trait Input<T: PacketType>: Send + Sync {
    async fn get(&self) -> Result<T, std::io::Error>;
}

/// A StateSwitcher serves the following purposes:
/// - Gather incoming packets from an [`Input`]
/// - Make the packet go through each successive state
/// while executing every defined [`Hook`] each time
/// - Dispatch the packet using an [`Output`]

pub struct StateSwitcher<T: PacketType + Send + 'static, U: PacketType + Send + 'static> {
    registry: Arc<HookRegistry<T, U>>, 
    output: Arc<Box<dyn Output<U>>>,
    input: Arc<Box<dyn Input<T>>>,
    dropped: Arc<AtomicUsize>,
    running: Arc<AtomicBool> 
}

unsafe impl<T: PacketType + Send, U: PacketType + Send> Sync for StateSwitcher<T, U> {}

impl<T: PacketType + Send, U: PacketType + Send> StateSwitcher<T, U> {


    /// Crates a new `StateSwitcher` from
    /// a [`HookRegistry`], an [`Input`] from which
    /// it will create the [`PacketContext`], and an [`Output`]
    /// to send the pakets that went through the [`Hook`]
    ///
    /// # Examples: 
    ///
    /// ```
    /// let state_switcher = StateSwitcher::new(input, output, registry);
    /// ```
    pub fn new(input: Box<dyn Input<T>>, output: Box<dyn Output<U>>, registry: HookRegistry<T, U>, kill_switch: Arc<AtomicBool>) -> Self {
        Self { registry: Arc::new(registry), output: Arc::new(output), input: Arc::new(input), dropped: Arc::new(AtomicUsize::new(0)), running: kill_switch }
    }

    /// Initiate the state switching process.
    /// Usually, it should be the main loop 
    /// of the program. 
    ///
    /// It gather incoming packet from the [`Input`]
    /// endlessly, make those packets go through
    /// the [`Hook`] and then send them to foreign
    /// devices using the [`Output`]
    ///
    /// # Examples:
    /// ```
    /// let state_switcher = StateSwitcher::new(input, output, registry);
    ///
    /// state_switcher.start().await;
    /// ```
    pub async fn start(&self) {

        loop {

            if !self.running.load(SeqCst) {
                break;
            }

            let packet = match self.input.get().await {
                Ok(pak) => pak,
                Err(_) => { continue; }
            };
            let mut context = PacketContext::from(packet);
            let registry = self.registry.clone();
            let output = self.output.clone();
            let drops = self.dropped.clone();
            
            tokio::spawn(async move { 

                for state in enum_iterator::all::<PacketState>().filter(|x| *x != PacketState::Failure) {
                    if state == PacketState::Failure {
                        continue;
                    }
                    context.set_state(state);
                    match registry.run_hooks(&mut context) {
                        Ok(_) => (),
                        Err(_) => {
                            drops.store(drops.load(SeqCst) + 1, SeqCst); 
                        }
                    };
                }
                    
                let output_packet = context.drop();
                let bytes_len = output_packet.to_raw_bytes().len();
                let success = output.send(output_packet)
                    .await
                    .ok()
                    .map(|len| { len == bytes_len })
                    .unwrap_or(false);

                if !success {
                    drops.store(drops.load(SeqCst) + 1, SeqCst);
                }
            });

        }   

    }

    /// Returns the number of packet dropped
    /// either through unsuccessful fatal [`Hook`]
    /// execution, or at the output.

    pub fn drop_count(&self) -> usize {
        self.dropped.load(SeqCst)
    }

} 

#[cfg(test)]
mod tests {

    use std::time::Duration;
    use tokio::time::sleep;

    use crate::hooks::{hook_registry::{HookClosure, Hook}, flags::HookFlag};

    use super::*;

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
            if packet.name == 2 {
                Ok(1)
            }
            else {
                Ok(0)
            }
        } 
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_exec_stack() {
        let mut registry: HookRegistry<A, A> = HookRegistry::new();
        registry.register_hook(PacketState::Received, Hook::new(String::from("test_hook"), HookClosure(Box::new(|_, packet: &mut PacketContext<A, A>| {
            packet.get_mut_output().name = 2;
            Ok(1)
        })), Vec::default()));
        let input = SimpleInput{};
        let output = SimpleOutput{};

        let switch = Arc::new(AtomicBool::new(true));

        let state_switcher = StateSwitcher::new(Box::new(input), Box::new(output), registry, switch.clone());

        tokio::spawn(async move{
            std::thread::sleep(Duration::from_secs(1));
            switch.store(false, SeqCst);
        });
        state_switcher.start().await;

        assert!(state_switcher.drop_count() == 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_state_switching() {
        let mut registry: HookRegistry<A, A> = HookRegistry::new();
        registry.register_hook(PacketState::Received, Hook::new(String::from("test_hook"), HookClosure(Box::new(|_, packet: &mut PacketContext<A, A>| {
            packet.get_mut_output().name = 5;
            Ok(1)
        })), vec![HookFlag::Fatal]));
        registry.register_hook(PacketState::Prepared, Hook::new(String::from("test_hook"), HookClosure(Box::new(|_, packet: &mut PacketContext<A, A>| {
            assert!(packet.get_output().name == 5);
            packet.get_mut_output().name = 2;
            Ok(1)
        })), vec![HookFlag::Fatal]));
        let input = SimpleInput{};
        let output = SimpleOutput{};

        let switch = Arc::new(AtomicBool::new(true));
        let state_switcher = StateSwitcher::new(Box::new(input), Box::new(output), registry, switch.clone());

        tokio::spawn(async move {
            sleep(Duration::from_secs(1)).await;
            switch.store(false, SeqCst);
        });
        state_switcher.start().await;

        assert!(state_switcher.drop_count() == 0);
    }

    
        
}
