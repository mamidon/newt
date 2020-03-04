use crate::DrawCommandKind;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};

#[derive(Default, Debug, Clone)]
pub struct GlyphVertex {
    pub position: [f32; 2],
    pub uv_input: [f32; 2],
}
vulkano::impl_vertex!(GlyphVertex, position, uv_input);

#[derive(Clone)]
pub(crate) struct GlyphPipeline {
    inner: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

impl GlyphPipeline {
    pub fn create_pipeline(
        device: Arc<Device>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    ) -> GlyphPipeline {
        let vs = vertex_shader::Shader::load(device.clone()).unwrap();
        let fs = fragment_shader::Shader::load(device.clone()).unwrap();

        let inner = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<GlyphVertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(Subpass::from(render_pass, 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        GlyphPipeline { inner }
    }

    pub fn bind(&self, kind: DrawCommandKind) -> () {
        let sampler = Sampler::new(
            self.inner.device().clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0,
            1.0,
            0.0,
            0.0,
        )
        .expect("Sampler::new failed");
        let surface = unimplemented!("Fetching the actual GPU surface from the resources table");

        unimplemented!("Actually binding the resources into a PersistentDescriptorSet")
    }
}

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r#"
    #version 450

    layout(location = 0) in vec2 position;
    layout(location = 1) in vec2 uv_input;
    
    layout(location = 0) out vec2 uv_output;
    
    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
        uv_output = uv_input;
    }
    "#
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r#"
    #version 450
    
    layout(location = 0) in vec2 uv_in;
    
    layout(location = 0) out vec4 color_out;

    void main() {
        color_out = vec4(uv_in, 0.0, 0.0);
    }
    "#
    }
}
