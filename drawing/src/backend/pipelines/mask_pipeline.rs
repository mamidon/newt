use crate::backend::{GpuFrame, MaskDrawData};
use crate::{DrawingResult, Extent, MaskId, ResourceTable};
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::DescriptorSet;
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};

#[derive(Default, Debug, Clone)]
pub struct MaskVertex {
    pub position: [f32; 2],
    pub uv_input: [f32; 2],
    pub foreground_input: u32,
    pub background_input: u32,
}
vulkano::impl_vertex!(
    MaskVertex,
    position,
    uv_input,
    foreground_input,
    background_input
);

#[derive(Clone)]
pub(crate) struct MaskPipeline {
    inner: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    resource_table: Arc<ResourceTable>,
}

impl MaskPipeline {
    pub fn create_pipeline(
        device: Arc<Device>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        resource_table: Arc<ResourceTable>,
    ) -> MaskPipeline {
        let vs = vertex_shader::Shader::load(device.clone()).unwrap();
        let fs = fragment_shader::Shader::load(device.clone()).unwrap();

        let inner = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<MaskVertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(Subpass::from(render_pass, 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        MaskPipeline {
            inner,
            resource_table,
        }
    }

    pub fn write_commands(
        &self,
        frame: &GpuFrame,
        masks: &HashMap<MaskId, Vec<MaskDrawData>>,
        mut builder: AutoCommandBufferBuilder,
    ) -> DrawingResult<AutoCommandBufferBuilder> {
        for (mask_id, data) in masks.iter() {
            let mut mask_vertices: Vec<MaskVertex> = Vec::new();

            let binding = self.bind(*mask_id);

            for datum in data.iter() {
                let MaskDrawData { brush, extent } = datum;

                extent
                    .corners()
                    .map(|corner| [corner[0] as f32, corner[1] as f32])
                    .map(|corner| {
                        [
                            corner[0] * 2.0 / frame.target_dimensions[0] as f32 - 1.0,
                            corner[1] * 2.0 / frame.target_dimensions[1] as f32 - 1.0,
                        ]
                    })
                    .enumerate()
                    .for_each(|(index, corner)| {
                        mask_vertices.push(MaskVertex {
                            position: corner,
                            uv_input: Extent::uv_coordinates(index),
                            foreground_input: brush.foreground,
                            background_input: brush.background,
                        })
                    });
            }

            let vertex_buffer: Vec<Arc<dyn BufferAccess + Send + Sync + 'static>> = {
                vec![CpuAccessibleBuffer::from_iter(
                    self.inner.device().clone(),
                    BufferUsage::all(),
                    mask_vertices.into_iter(),
                )
                .unwrap()]
            };

            builder = builder
                .draw(
                    self.inner.clone(),
                    &frame.dynamic_state,
                    vertex_buffer,
                    binding.clone(),
                    (),
                )
                .expect("Failed to draw mask")
        }

        Ok(builder)
    }

    fn bind(&self, mask_id: MaskId) -> Arc<dyn DescriptorSet + Send + Sync> {
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

        let surface = self.resource_table.get_mask(mask_id);

        Arc::new(
            PersistentDescriptorSet::start(self.inner.clone(), 0)
                .add_sampled_image(surface.gpu_surface, sampler.clone())
                .expect("add_sampled_image failed")
                .build()
                .expect("build persistent_descriptor_set failed"),
        )
    }
}

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r#"
    #version 450

    layout(location = 0) in vec2 position;
    layout(location = 1) in vec2 uv_input;
    layout(location = 2) in uint foreground_input;
    layout(location = 3) in uint background_input;
    
    layout(location = 0) out vec2 uv_output;
    layout(location = 1) out uint foreground_output;
    layout(location = 2) out uint background_output;
    
    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
        uv_output = uv_input;
        foreground_output = foreground_input;
        background_output = background_input;
    }
    "#
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r#"
    #version 450
    layout(location = 0) in vec2 tex_coords;
    layout(location = 1) flat in uint foreground;
    layout(location = 2) flat in uint background;
    
    layout(location = 0) out vec4 f_color;

    layout(set = 0, binding = 0) uniform usampler2D tex;

    void main() {
        uvec4 scale4 = texture(tex, tex_coords);
        float scale = float(scale4.r) / 255.0;
        
        vec4 a = unpackUnorm4x8(foreground);
        vec4 b = unpackUnorm4x8(background);
        vec4 lerped = mix(b, a, scale);
        
        //unpackUnorm4x8 reverses the endianness
        f_color = vec4(lerped.a, lerped.b, lerped.g, lerped.r); 
    }
    "#
    }
}
