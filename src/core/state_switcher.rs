use std::sync::Arc;

use async_trait::async_trait;
use log::trace;

use crate::hooks::hook_registry::HookRegistry;

use super::packet::{PacketType, PacketContext};

#[async_trait]
pub trait Output<T: PacketType>: Send + Sync {
    async fn send(&self, packet: &T);
}

pub trait Input<T: PacketType>: Send {
    fn get(&self) -> T;
}

pub struct StateSwitcher<T: PacketType + Send + 'static, U: PacketType + Send + 'static> {
    registry: Arc<&'static mut HookRegistry<T, U>>, 
    output: Arc<&'static dyn Output<U>>,
    input: &'static dyn Input<T>,
}

unsafe impl<T: PacketType + Send, U: PacketType + Send> Sync for StateSwitcher<T, U> {}

impl<T: PacketType + Send, U: PacketType + Send> StateSwitcher<T, U> {

    pub fn new(input: &'static dyn Input<T>, output: &'static dyn Output<U>, registry: &'static mut HookRegistry<T, U>) -> Self {
        Self { registry: Arc::new(registry), output: Arc::new(output), input}
    }

    pub async fn start(&self) -> Result<(), ()> {

        loop {

            let packet = self.input.get();
            let mut context = self.prepare_packet(packet);
            let registry = self.registry.clone();
            let output = self.output.clone();
            
            tokio::spawn(async move { 
                registry.run_hooks(&mut context);
                output.send(context.get_output());
            });

        }   

    }

    fn prepare_packet(&self, packet: T) -> PacketContext<T, U>{

        PacketContext::from(packet)

    }
} 

#[cfg(test)]
mod tests {


        
}
