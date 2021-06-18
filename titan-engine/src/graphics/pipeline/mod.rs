use std::error::Error;

use ::slotmap::Key as SlotMapKey;
use ash::version::DeviceV1_0;
use ash::vk;

pub use layout::PipelineLayout;
use proc_macro::SlotMappable;
pub use render_pass::RenderPass;

use super::{
    device::Device,
    shader,
    shader::{ShaderModule, FRAG_SHADER_CODE, VERT_SHADER_CODE},
    slotmap::SlotMappable,
    swapchain::Swapchain,
    utils,
};

pub mod layout;
pub mod render_pass;

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct GraphicsPipeline {
    key: Key,
    handle: vk::Pipeline,
    parent_render_pass: render_pass::Key,
    parent_pipeline_layout: layout::Key,
}

impl GraphicsPipeline {
    pub fn new(
        key: Key,
        render_pass_key: render_pass::Key,
        pipeline_layout_key: layout::Key,
    ) -> Result<Self, Box<dyn Error>> {
        let slotmap_pipeline_layout = PipelineLayout::slotmap().read()?;
        let pipeline_layout = slotmap_pipeline_layout
            .get(pipeline_layout_key)
            .ok_or_else(|| utils::make_error("pipeline layout not found"))?;

        let slotmap_render_pass = RenderPass::slotmap().read()?;
        let render_pass = slotmap_render_pass
            .get(render_pass_key)
            .ok_or_else(|| utils::make_error("render pass not found"))?;

        let swapchain_key = render_pass.parent_swapchain();
        let slotmap_swapchain = Swapchain::slotmap().read()?;
        let render_pass_swapchain = slotmap_swapchain
            .get(swapchain_key)
            .ok_or_else(|| utils::make_error("swapchain not found"))?;

        let render_pass_device = render_pass_swapchain.parent_device();
        let pipeline_layout_device = pipeline_layout.parent_device();
        if render_pass_device != pipeline_layout_device {
            return Err(utils::make_error(
                "pipeline layout and render pass must have the same parent",
            )
            .into());
        }
        let device_key = render_pass_device;
        let slotmap_device = Device::slotmap().read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;

        let vert_shader_module =
            ShaderModule::new(shader::Key::null(), device_key, VERT_SHADER_CODE)?;
        let frag_shader_module =
            ShaderModule::new(shader::Key::null(), device_key, FRAG_SHADER_CODE)?;

        let shader_stage_info_name = crate::c_str!("main");
        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module.handle())
            .name(shader_stage_info_name);
        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module.handle())
            .name(shader_stage_info_name);
        let shader_stage_infos = [*vert_shader_stage_info, *frag_shader_stage_info];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder();
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: render_pass_swapchain.extent().width as f32,
            height: render_pass_swapchain.extent().height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let viewports = [viewport];
        let scissor = vk::Rect2D::builder().extent(render_pass_swapchain.extent());
        let scissors = [*scissor];
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);
        let attachments = [*color_blend_attachment];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&attachments);

        let create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_infos)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout.handle())
            .render_pass(render_pass.handle())
            .subpass(0)
            .base_pipeline_index(-1);
        let create_infos = [*create_info];
        let handles = unsafe {
            device.loader().create_graphics_pipelines(
                vk::PipelineCache::default(),
                &create_infos,
                None,
            )
        };
        let handle = handles
            .map(|handles| {
                handles
                    .into_iter()
                    .next()
                    .ok_or_else(|| utils::make_error("graphics pipeline was not created"))
            })
            .map_err(|_| utils::make_error("graphics pipeline was not created"))??;
        Ok(Self {
            key,
            handle,
            parent_render_pass: render_pass_key,
            parent_pipeline_layout: pipeline_layout_key,
        })
    }

    pub fn handle(&self) -> vk::Pipeline {
        self.handle
    }

    pub fn parent_render_pass(&self) -> render_pass::Key {
        self.parent_render_pass
    }

    pub fn parent_pipeline_layout(&self) -> layout::Key {
        self.parent_pipeline_layout
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        let slotmap_render_pass = match RenderPass::slotmap().read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let render_pass = match slotmap_render_pass.get(self.parent_render_pass()) {
            None => return,
            Some(value) => value,
        };

        let slotmap_swapchain = match Swapchain::slotmap().read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let swapchain = match slotmap_swapchain.get(render_pass.parent_swapchain()) {
            None => return,
            Some(value) => value,
        };

        let slotmap_device = match Device::slotmap().read() {
            Ok(value) => value,
            Err(_) => return,
        };
        let device = match slotmap_device.get(swapchain.parent_device()) {
            None => return,
            Some(value) => value,
        };
        unsafe { device.loader().destroy_pipeline(self.handle, None) }
    }
}
