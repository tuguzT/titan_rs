use std::error::Error;

use ash::version::DeviceV1_0;
use ash::vk;

use super::slotmap::{DeviceKey, SLOTMAP_DEVICE};
use super::utils;

pub struct Framebuffer {
    handle: vk::Framebuffer,
    parent_device: DeviceKey,
}

impl Framebuffer {
    pub unsafe fn new(
        device_key: DeviceKey,
        create_info: &vk::FramebufferCreateInfo,
    ) -> Result<Self, Box<dyn Error>> {
        let slotmap_device = SLOTMAP_DEVICE.read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

        let handle = device.loader().create_framebuffer(create_info, None)?;
        Ok(Self {
            handle,
            parent_device: device_key,
        })
    }

    pub fn handle(&self) -> vk::Framebuffer {
        self.handle
    }

    pub fn parent_device(&self) -> DeviceKey {
        self.parent_device
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        let slotmap_device = match SLOTMAP_DEVICE.read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let device = match slotmap_device.get(self.parent_device()) {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_framebuffer(self.handle, None) }
    }
}
