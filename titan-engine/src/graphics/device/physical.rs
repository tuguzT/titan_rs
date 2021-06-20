use std::cmp::Ordering;
use std::error::Error;
use std::ffi::CStr;

use ash::prelude::VkResult;
use ash::version::InstanceV1_0;
use ash::vk;

use proc_macro::SlotMappable;

use super::super::{
    instance::{self, Instance},
    slotmap::SlotMappable,
    surface::Surface,
    utils,
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct PhysicalDevice {
    key: Key,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    queue_family_properties: Vec<vk::QueueFamilyProperties>,
    layer_properties: Vec<vk::LayerProperties>,
    extension_properties: Vec<vk::ExtensionProperties>,
    handle: vk::PhysicalDevice,
    parent_instance: instance::Key,
}

impl PhysicalDevice {
    pub unsafe fn new(
        instance_key: instance::Key,
        handle: vk::PhysicalDevice,
    ) -> Result<Key, Box<dyn Error>> {
        let slotmap_instance = SlotMappable::slotmap().read().unwrap();
        let instance: &Instance = slotmap_instance
            .get(instance_key)
            .expect("instance not found");

        let properties = instance.loader().get_physical_device_properties(handle);
        let features = instance.loader().get_physical_device_features(handle);
        let memory_properties = instance
            .loader()
            .get_physical_device_memory_properties(handle);
        let queue_family_properties = instance
            .loader()
            .get_physical_device_queue_family_properties(handle);
        let layer_properties = enumerate_device_layer_properties(instance.loader(), handle)?;
        let extension_properties = instance
            .loader()
            .enumerate_device_extension_properties(handle)?;

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle,
            properties,
            features,
            queue_family_properties,
            memory_properties,
            layer_properties,
            extension_properties,
            parent_instance: instance_key,
        });
        Ok(key)
    }

    pub fn handle(&self) -> ash::vk::PhysicalDevice {
        self.handle
    }

    pub fn parent_instance(&self) -> instance::Key {
        self.parent_instance
    }

    pub fn is_suitable(&self) -> bool {
        let mut graphics_queue_family_properties = self
            .queue_family_properties_with(vk::QueueFlags::GRAPHICS)
            .peekable();
        let mut extension_properties_names =
            self.extension_properties
                .iter()
                .map(|extension_property| unsafe {
                    CStr::from_ptr(extension_property.extension_name.as_ptr())
                });
        let has_required_extensions = super::REQUIRED_EXTENSIONS.iter().any(|required_name| {
            extension_properties_names
                .find(|item| item == required_name)
                .is_some()
        });
        graphics_queue_family_properties.peek().is_some() && has_required_extensions
    }

    pub fn score(&self) -> u32 {
        let mut score = match self.properties.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => 1000,
            vk::PhysicalDeviceType::INTEGRATED_GPU => 100,
            _ => 0,
        };
        score += self.properties.limits.max_image_dimension2_d;
        score
    }

    pub fn queue_family_properties(&self) -> &Vec<vk::QueueFamilyProperties> {
        &self.queue_family_properties
    }

    pub fn queue_family_properties_with(
        &self,
        flags: vk::QueueFlags,
    ) -> impl Iterator<Item = (usize, &vk::QueueFamilyProperties)> {
        self.queue_family_properties.iter().enumerate().filter(
            move |(_index, queue_family_properties)| {
                let ref inner_flags = queue_family_properties.queue_flags;
                inner_flags.contains(flags)
            },
        )
    }

    pub fn layer_properties(&self) -> &Vec<vk::LayerProperties> {
        &self.layer_properties
    }

    pub fn extension_properties(&self) -> &Vec<vk::ExtensionProperties> {
        &self.extension_properties
    }

    pub fn graphics_family_index(&self) -> Result<u32, Box<dyn Error>> {
        let graphics_queue_family_properties =
            self.queue_family_properties_with(vk::QueueFlags::GRAPHICS);
        let graphics_family_index = graphics_queue_family_properties
            .peekable()
            .peek()
            .ok_or_else(|| utils::make_error("no queues with graphics support"))?
            .0 as u32;
        Ok(graphics_family_index)
    }

    pub fn present_family_index(&self, surface: &Surface) -> Result<u32, Box<dyn Error>> {
        let present_queue_family_properties =
            surface.physical_device_queue_family_properties_support(&self);
        let present_family_index = present_queue_family_properties
            .peekable()
            .peek()
            .ok_or_else(|| utils::make_error("no queues with surface present support"))?
            .0 as u32;
        Ok(present_family_index)
    }
}

impl PartialEq for PhysicalDevice {
    fn eq(&self, other: &Self) -> bool {
        self.score().eq(&other.score())
    }
}

impl PartialOrd for PhysicalDevice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score().partial_cmp(&other.score())
    }
}

impl Eq for PhysicalDevice {}

impl Ord for PhysicalDevice {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score().cmp(&other.score())
    }
}

unsafe fn enumerate_device_layer_properties(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> VkResult<Vec<vk::LayerProperties>> {
    let mut count = 0;
    instance
        .fp_v1_0()
        .enumerate_device_layer_properties(physical_device, &mut count, std::ptr::null_mut())
        .result()?;
    let mut data = Vec::with_capacity(count as usize);
    let err_code = instance.fp_v1_0().enumerate_device_layer_properties(
        physical_device,
        &mut count,
        data.as_mut_ptr(),
    );
    data.set_len(count as usize);
    err_code.result_with_success(data)
}