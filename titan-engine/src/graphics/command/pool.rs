use std::error::Error;

use ash::version::DeviceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use super::super::{
    command::{self, CommandBuffer},
    device::{self, Device},
    slotmap::SlotMappable,
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct CommandPool {
    key: Key,
    handle: vk::CommandPool,
    parent_device: device::Key,
}

impl CommandPool {
    pub unsafe fn new(
        device_key: device::Key,
        create_info: &vk::CommandPoolCreateInfo,
    ) -> Result<Key, Box<dyn Error>> {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");
        let handle = device.loader().create_command_pool(create_info, None)?;

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            parent_device: device_key,
        });
        Ok(key)
    }

    pub fn handle(&self) -> vk::CommandPool {
        self.handle
    }

    pub fn parent_device(&self) -> device::Key {
        self.parent_device
    }

    pub fn enumerate_command_buffers(
        &self,
        count: u32,
    ) -> Result<Vec<command::buffer::Key>, Box<dyn Error>> {
        let device_key = self.parent_device();
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("parent was lost");

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.handle())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);
        unsafe {
            device
                .loader()
                .allocate_command_buffers(&allocate_info)?
                .into_iter()
                .map(|command_buffer| CommandBuffer::new(self.key, command_buffer))
                .collect()
        }
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device
            .get(self.parent_device())
            .expect("device not found");
        unsafe { device.loader().destroy_command_pool(self.handle, None) }
    }
}