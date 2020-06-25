use crate::backend::{GpuFrame, ShapeDrawData};
use crate::{Color, DrawCommandKind, DrawList, DrawingResult, Extent, ShapeKind};
use std::sync::Arc;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};

#[derive(Default, Debug, Clone)]
struct ShapeVertex {
    position: [f32; 2],
    uv_input: [f32; 2],
    kind_input: i32,
    foreground_input: [f32; 4],
    background_input: [f32; 4],
}
vulkano::impl_vertex!(
    ShapeVertex,
    position,
    uv_input,
    kind_input,
    foreground_input,
    background_input
);

#[derive(Clone)]
pub(crate) struct ShapePipeline {
    inner: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

impl ShapePipeline {
    pub fn create_pipeline(
        device: Arc<Device>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    ) -> ShapePipeline {
        let vs = vertex_shader::Shader::load(device.clone()).unwrap();
        let fs = fragment_shader::Shader::load(device.clone()).unwrap();

        let inner = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<ShapeVertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(Subpass::from(render_pass, 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        ShapePipeline { inner }
    }

    pub fn write_commands(
        &self,
        frame: &GpuFrame,
        draw_data: &Vec<ShapeDrawData>,
        builder: AutoCommandBufferBuilder,
    ) -> DrawingResult<AutoCommandBufferBuilder> {
        let mut shape_vertices: Vec<ShapeVertex> = Vec::new();
        for data in draw_data {
            let kind = data.kind;
            let brush = data.brush;
            let extent = data.extent;

            let kind_input = match kind {
                ShapeKind::Rectangle => 1,
                ShapeKind::Ellipse => 2,
                ShapeKind::Line => 3,
            };

            let corners: Vec<[f32; 2]> = extent
                .corners()
                .map(|corner| [corner[0] as f32, corner[1] as f32])
                .map(|corner| {
                    [
                        corner[0] * 2.0 / frame.target_dimensions[0] as f32 - 1.0,
                        corner[1] * 2.0 / frame.target_dimensions[1] as f32 - 1.0,
                    ]
                })
                .collect();

            for (index, corner) in corners.iter().enumerate() {
                shape_vertices.push(ShapeVertex {
                    kind_input,
                    position: *corner,
                    uv_input: Extent::logical_device_coordinates(index),
                    foreground_input: self.to_color(brush.foreground),
                    background_input: self.to_color(brush.background),
                })
            }
        }

        let vertex_buffer: Vec<Arc<dyn BufferAccess + Send + Sync + 'static>> = {
            vec![CpuAccessibleBuffer::from_iter(
                self.inner.device().clone(),
                BufferUsage::all(),
                shape_vertices.into_iter(),
            )
            .unwrap()]
        };

        builder
            .draw(
                self.inner.clone(),
                &frame.dynamic_state,
                vertex_buffer,
                (),
                (),
            )
            .map_err(|_| "Failed to draw shapes")
    }

    fn to_color(&self, color: Color) -> [f32; 4] {
        let red = ((color & 0xFF000000) >> 24) as f32 / 255.0;
        let green = ((color & 0x00FF0000) >> 16) as f32 / 255.0;
        let blue = ((color & 0x0000FF00) >> 8) as f32 / 255.0;
        let alpha = (color & 0x000000FF) as f32 / 255.0;
        return [red, green, blue, alpha];
    }
}

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r#"
    #version 450

    layout(location = 0) in vec2 position;
    layout(location = 1) in vec2 uv_input;
    layout(location = 2) in int kind_input;
    layout(location = 3) in vec4 foreground_input;
    layout(location = 4) in vec4 background_input;
    
    layout(location = 0) out vec2 uv_output;
    layout(location = 1) out int kind_out;
    layout(location = 2) out vec4 foreground_out;
    layout(location = 3) out vec4 background_out;
    
    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
        uv_output = uv_input;
        kind_out = kind_input;
        foreground_out = foreground_input;
        background_out = background_input;
    }
    "#
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r#"
    #version 450

    #define KIND_RECTANGLE (1)
    #define KIND_CIRCLE (2)
    
    layout(location = 0) in vec2 uv_in;
    layout(location = 1) flat in int kind_in;
    layout(location = 2) flat in vec4 foreground_in;
    layout(location = 3) flat in vec4 background_in;
    
    layout(location = 0) out vec4 color_out;

    void main() {
        if (kind_in == KIND_RECTANGLE) {
            if (abs(uv_in.x) < 1.0 && abs(uv_in.y) < 1.0) {
                color_out = foreground_in;
            } else {
                color_out = background_in;
            }
        } else if (kind_in == KIND_CIRCLE) {
            if (abs(uv_in.x)*abs(uv_in.x) + abs(uv_in.y)*abs(uv_in.y) < 1.0) {
                color_out = foreground_in;
            } else {
                color_out = background_in;
            }
        } else {
            color_out = vec4(1.0, 0.0, 1.0, 1.0);
        }
    }
    "#
    }
}
