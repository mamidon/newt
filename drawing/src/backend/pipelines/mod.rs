use crate::backend::pipelines::shape_pipeline::ShapeVertex;
use crate::backend::GpuFrame;
use crate::{Color, DrawList, ShapeDrawData, ShapeKind};
use std::sync::Arc;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::Device;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::pipeline::GraphicsPipelineAbstract;

mod glyph_pipeline;
mod shape_pipeline;

type OwnedRenderPass = Arc<dyn RenderPassAbstract + Send + Sync>;
type OwnedGraphicsPipeline = Arc<dyn GraphicsPipelineAbstract + Send + Sync>;

#[derive(Clone)]
pub struct GpuPipelines {
    shapes: OwnedGraphicsPipeline,
    glyphs: OwnedGraphicsPipeline,
}

impl GpuPipelines {
    pub fn new(device: Arc<Device>, render_pass: OwnedRenderPass) -> GpuPipelines {
        GpuPipelines {
            shapes: shape_pipeline::create_pipeline(device.clone(), render_pass.clone()),
            glyphs: glyph_pipeline::create_pipeline(device.clone(), render_pass.clone()),
        }
    }

    pub fn write_commands(
        &self,
        target_width: f32,
        target_height: f32,
        dynamic_state: &DynamicState,
        mut command_buffer_builder: AutoCommandBufferBuilder,
        draw_list: &DrawList,
    ) -> AutoCommandBufferBuilder {
        let mut shape_vertices: Vec<ShapeVertex> = Vec::new();
        for shape in draw_list.shapes.iter() {
            let ShapeDrawData {
                brush,
                extent,
                kind,
            } = shape;

            let left = extent.x as f32 / target_width * 2.0 - 1.0;
            let right = (extent.x + extent.width as i64) as f32 / target_width * 2.0 - 1.0;
            let top = extent.y as f32 / target_height * 2.0 - 1.0;
            let bottom = (extent.y + extent.height as i64) as f32 / target_height * 2.0 - 1.0;

            let kind_input = match kind {
                ShapeKind::Rectangle => 1,
                ShapeKind::Ellipse => 2,
                ShapeKind::Line => 3,
            };

            shape_vertices.push(ShapeVertex {
                position: [left, top],
                uv_input: [-1.0, -1.0],
                kind_input,
                foreground_input: self.to_color(brush.foreground),
                background_input: self.to_color(brush.background),
            });
            shape_vertices.push(ShapeVertex {
                position: [right, top],
                uv_input: [1.0, -1.0],
                kind_input,
                foreground_input: self.to_color(brush.foreground),
                background_input: self.to_color(brush.background),
            });
            shape_vertices.push(ShapeVertex {
                position: [left, bottom],
                uv_input: [-1.0, 1.0],
                kind_input,
                foreground_input: self.to_color(brush.foreground),
                background_input: self.to_color(brush.background),
            });
            shape_vertices.push(ShapeVertex {
                position: [right, top],
                uv_input: [1.0, -1.0],
                kind_input,
                foreground_input: self.to_color(brush.foreground),
                background_input: self.to_color(brush.background),
            });
            shape_vertices.push(ShapeVertex {
                position: [right, bottom],
                uv_input: [1.0, 1.0],
                kind_input,
                foreground_input: self.to_color(brush.foreground),
                background_input: self.to_color(brush.background),
            });
            shape_vertices.push(ShapeVertex {
                position: [left, bottom],
                uv_input: [-1.0, 1.0],
                kind_input,
                foreground_input: self.to_color(brush.foreground),
                background_input: self.to_color(brush.background),
            });
        }

        let vertex_buffer: Vec<Arc<dyn BufferAccess + Send + Sync + 'static>> = {
            vec![CpuAccessibleBuffer::from_iter(
                self.shapes.device().clone(),
                BufferUsage::all(),
                shape_vertices.into_iter(),
            )
            .unwrap()]
        };

        command_buffer_builder = command_buffer_builder
            .draw(self.shapes.clone(), &dynamic_state, vertex_buffer, (), ())
            .expect("Failed to draw shapes");

        command_buffer_builder
    }

    fn to_color(&self, color: Color) -> [f32; 4] {
        let red = ((color & 0xFF000000) >> 24) as f32 / 255.0;
        let green = ((color & 0x00FF0000) >> 16) as f32 / 255.0;
        let blue = ((color & 0x0000FF00) >> 8) as f32 / 255.0;
        let alpha = (color & 0x000000FF) as f32 / 255.0;
        return [red, green, blue, alpha];
    }
}
