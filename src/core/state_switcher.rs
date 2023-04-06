//! State switching logic implementation for [`PacketContext`]
//! A `StateSwitcher` runs all hooks defined for a given
//! state before transitioning to the next state
//!
//! `Input` and `Output` are abstract types
//! used to gather incoming data and dispatch
//! outgoing one.

use std::sync::{Arc, atomic::{AtomicUsize, Ordering::SeqCst}};

use async_trait::async_trait;
use crate::hooks::hook_registry::HookRegistry;

use super::{packet::{PacketType, PacketContext}, state::PacketState};

#[async_trait]
pub trait Output<T: PacketType>: Send + Sync {
    async fn send(&self, packet: T) -> Result<usize, std::io::Error>;
}

#[async_trait]
pub trait Input<T: PacketType>: Send {
    async fn get(&self) -> Result<T, std::io::Error>;
}

/// A StateSwitcher serves the following purposes:
/// - Gather incoming packets from an [`Input`]
/// - Make the packet go through each successive state
/// while executing every defined [`Hook`] each time
/// - Dispatch the packet using an [`Output`]

pub struct StateSwitcher<T: PacketType + Send + 'static, U: PacketType + Send + 'static> {
    registry: Arc<&'static mut HookRegistry<T, U>>, 
    output: Arc<&'static dyn Output<U>>,
    input: &'static dyn Input<T>,
    dropped: Arc<AtomicUsize>
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
    pub fn new(input: &'static dyn Input<T>, output: &'static dyn Output<U>, registry: &'static mut HookRegistry<T, U>) -> Self {
        Self { registry: Arc::new(registry), output: Arc::new(output), input, dropped: Arc::new(AtomicUsize::new(0))}
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

            let packet = match self.input.get().await {
                Ok(pak) => pak,
                Err(_) => { continue; }
            };
            let mut context = PacketContext::from(packet);
            let registry = self.registry.clone();
            let output = self.output.clone();
            let drops = self.dropped.clone();

            
            tokio::spawn(async move { 

                for state in enum_iterator::all::<PacketState>() {
                    context.set_state(state);
                    registry.run_hooks(&mut context);
                }
                    
                let output_packet = context.drop();
                let bytes_len = output_packet.to_raw_bytes().len();
                let success = output.send(output_packet)
                    .await
                    .map(|len| { len == bytes_len })
                    .is_ok();

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

        
        
}
