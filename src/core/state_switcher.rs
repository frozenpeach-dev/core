use std::sync::{Arc, atomic::{AtomicUsize, Ordering::SeqCst}};

use async_trait::async_trait;

use crate::hooks::hook_registry::HookRegistry;

use super::packet::{PacketType, PacketContext};

#[async_trait]
pub trait Output<T: PacketType>: Send + Sync {
    async fn send(&self, packet: T) -> Result<usize, std::io::Error>;
}

#[async_trait]
pub trait Input<T: PacketType>: Send {
    async fn get(&self) -> Result<T, std::io::Error>;
}

pub struct StateSwitcher<T: PacketType + Send + 'static, U: PacketType + Send + 'static> {
    registry: Arc<&'static mut HookRegistry<T, U>>, 
    output: Arc<&'static dyn Output<U>>,
    input: &'static dyn Input<T>,
    dropped: Arc<AtomicUsize>
}

unsafe impl<T: PacketType + Send, U: PacketType + Send> Sync for StateSwitcher<T, U> {}

impl<T: PacketType + Send, U: PacketType + Send> StateSwitcher<T, U> {

    pub fn new(input: &'static dyn Input<T>, output: &'static dyn Output<U>, registry: &'static mut HookRegistry<T, U>) -> Self {
        Self { registry: Arc::new(registry), output: Arc::new(output), input, dropped: Arc::new(AtomicUsize::new(0))}
    }

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
                registry.run_hooks(&mut context);
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

} 

#[cfg(test)]
mod tests {


        
}
