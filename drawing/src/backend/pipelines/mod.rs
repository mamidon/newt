use crate::backend::pipelines::glyph_pipeline::GlyphPipeline;
use crate::backend::pipelines::shape_pipeline::ShapePipeline;
use crate::backend::GpuFrame;
use crate::{Color, DrawList, ResourceTable, ShapeDrawData, ShapeKind};
use std::sync::Arc;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::device::Device;
use vulkano::framebuffer::RenderPassAbstract;

mod glyph_pipeline;
mod shape_pipeline;

#[derive(Clone)]
pub(crate) struct GpuPipelines {
    shapes: ShapePipeline,
    glyphs: GlyphPipeline,
}

type OwnedRenderPass = Arc<dyn RenderPassAbstract + Send + Sync>;

impl GpuPipelines {
    pub fn new(
        device: Arc<Device>,
        render_pass: OwnedRenderPass,
        resource_table: Arc<ResourceTable>,
    ) -> GpuPipelines {
        GpuPipelines {
            shapes: ShapePipeline::create_pipeline(device.clone(), render_pass.clone()),
            glyphs: GlyphPipeline::create_pipeline(
                device.clone(),
                render_pass.clone(),
                resource_table.clone(),
            ),
        }
    }

    pub(crate) fn write_commands(
        &self,
        frame: &GpuFrame,
        draw_list: &DrawList,
    ) -> AutoCommandBuffer {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(
            frame.device.clone(),
            frame.graphics_queue.family(),
        )
        .expect("Failed to create command buffer")
        .begin_render_pass(
            frame.target.clone(),
            false,
            vec![[1.0, 1.0, 0.5, 1.0].into()],
        )
        .expect("Failed to begin render pass");

        command_buffer_builder = self
            .shapes
            .write_commands(frame, draw_list, command_buffer_builder)
            .expect("Failed to write to command buffer");

        command_buffer_builder = self
            .glyphs
            .write_commands(frame, draw_list, command_buffer_builder)
            .expect("");

        command_buffer_builder
            .end_render_pass()
            .expect("Failed to end_render_pass")
            .build()
            .unwrap()
    }
}
