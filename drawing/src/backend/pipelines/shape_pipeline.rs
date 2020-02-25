use std::sync::Arc;
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};

#[derive(Default, Debug, Clone)]
pub struct ShapeVertex {
    pub position: [f32; 2],
    pub uv_input: [f32; 2],
    pub kind_input: i32,
    pub foreground_input: [f32; 4],
    pub background_input: [f32; 4],
}
vulkano::impl_vertex!(
    ShapeVertex,
    position,
    uv_input,
    kind_input,
    foreground_input,
    background_input
);

pub fn create_pipeline(
    device: Arc<Device>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
    let vs = vertex_shader::Shader::load(device.clone()).unwrap();
    let fs = fragment_shader::Shader::load(device.clone()).unwrap();

    Arc::new(
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
    )
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
