use crate::newt_render::pipelines::CommandBufferWritingInfo;
use crate::newt_render::{NewtSurface, RenderCommand};
use std::sync::Arc;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::pool::standard::StandardCommandPoolBuilder;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer, DynamicState};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::{DescriptorSet, PipelineLayoutAbstract};
use vulkano::device::Device;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::SwapchainImage;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode, BorderColor};
use vulkano::swapchain::Swapchain;
use winit::Window;

#[derive(Default, Debug, Clone)]
pub struct Vertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

pub struct BoundAttachments {
    descriptor: Arc<dyn DescriptorSet + Send + Sync>,
    vertices: Vec<Arc<dyn BufferAccess + Send + Sync + 'static>>,
}

pub struct GlyphPipeline {
    pub pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    pub render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pub dynamic_state: DynamicState,
    pub framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
}

impl GlyphPipeline {
    pub fn initialize(
        device: &Arc<Device>,
        format: vulkano::format::Format,
        images: &Vec<Arc<SwapchainImage<Window>>>,
    ) -> Result<Self, &'static str> {
        let vs = vertex_shader::Shader::load(device.clone()).unwrap();
        let fs = fragment_shader::Shader::load(device.clone()).unwrap();

        let render_pass: Arc<dyn RenderPassAbstract + Send + Sync> = Arc::new(
            vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: format,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            )
            .map_err(|_| "Failed to create the RenderPass")?,
        );

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            reference: None,
            compare_mask: None,
            scissors: None,
            write_mask: None,
        };

        let framebuffers = window_size_dependent_setup(images, &render_pass, &mut dynamic_state);

        Ok(GlyphPipeline {
            pipeline,
            dynamic_state,
            render_pass,
            framebuffers,
        })
    }

    pub fn reinitialize(&mut self, images: &Vec<Arc<SwapchainImage<Window>>>) {
        self.framebuffers =
            window_size_dependent_setup(images, &self.render_pass, &mut self.dynamic_state);
    }

    pub fn create_attachments<'a, I>(
        &self,
        info: &CommandBufferWritingInfo<'a, I>
    ) -> BoundAttachments
    where
        I: Iterator<Item = &'a RenderCommand> + Clone,
    {
        let vertices: Vec<Vertex> = info
            .commands
            .clone()
            .map(|c| match c {
                RenderCommand::NewtSurface {
                    x,
                    y,
                    width,
                    height,
                    surface
                } => {
                    let xf = screen_to_logical_device_coordinate(*x, info.logical_width);
                    let yf = screen_to_logical_device_coordinate(*y, info.logical_height);
                    let wf = screen_to_logical_device_coordinate(
                        (x + *width as isize),
                        info.logical_width,
                    );
                    let hf = screen_to_logical_device_coordinate(
                        (y + *height as isize),
                        info.logical_height,
                    );

                    let top_left = [xf, yf];
                    let top_right = [wf, yf];
                    let bottom_left = [xf, hf];
                    let bottom_right = [wf, hf];

                    vec![
                        Vertex { position: top_left },
                        Vertex {
                            position: top_right,
                        },
                        Vertex {
                            position: bottom_left,
                        },
                        Vertex {
                            position: top_right,
                        },
                        Vertex {
                            position: bottom_right,
                        },
                        Vertex {
                            position: bottom_left,
                        },
                    ]
                }
                _ => vec![],
            })
            .flat_map(|vertices| vertices.into_iter())
            .collect();

        let vertex_buffer: Vec<Arc<dyn BufferAccess + Send + Sync + 'static>> = {
            vec![CpuAccessibleBuffer::from_iter(
                self.pipeline.device().clone(),
                BufferUsage::all(),
                vertices.into_iter(),
            )
            .unwrap()]
        };

        let sampler = Sampler::new(
            self.pipeline.device().clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0,
            1.0,
            0.0,
            0.0
        ).expect("Sampler::new failed");
        let surface = info.commands.clone().
            filter_map(|c| match c {
            RenderCommand::NewtSurface {
                surface,
                x,
                y,
                width,
                height
            } => Some(surface),
            _ => None
        }).next().unwrap();

        let binding = self.bind_attachment(surface);

        BoundAttachments {
            descriptor: binding,
            vertices: vertex_buffer,
        }
    }

    fn bind_attachment(&self, surface: &NewtSurface) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = Sampler::new(
            self.pipeline.device().clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::ClampToBorder(BorderColor::FloatOpaqueBlack),
            SamplerAddressMode::ClampToBorder(BorderColor::FloatOpaqueBlack),
            SamplerAddressMode::ClampToBorder(BorderColor::FloatOpaqueBlack),
            0.0,
            1.0,
            0.0,
            0.0,
        )
        .unwrap();

        let ds = Arc::new(
            PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                .add_sampled_image(surface.0.clone(), sampler.clone())
                .expect("add_sampled_image failed")
                .build()
                .expect("build persistent_descriptor_set failed"),
        );

        ds
    }

    pub fn write_to_command_buffer<'a, I, P>(
        &self,
        info: &CommandBufferWritingInfo<'a, I>,
        builder: AutoCommandBufferBuilder<P>,
        attachments: BoundAttachments,
    ) -> Result<AutoCommandBufferBuilder<P>, &'static str>
    where
        I: Iterator<Item = &'a RenderCommand>,
    {
        builder
            .begin_render_pass(
                self.framebuffers[info.image_index].clone(),
                false,
                vec![[0.0, 0.0, 1.0, 1.0].into()],
            )
            .map_err(|_| "begin_render_pass failed")?
            .draw(
                self.pipeline.clone(),
                info.dynamic_state,
                attachments.vertices,
                attachments.descriptor.clone(),
                (),
            )
            .map_err(|e| {
                println!("{:?}", e);
                "draw failed"
            })?
            .end_render_pass()
            .map_err(|_| "end_render_pass failed")
    }

    pub fn map_commands_to_vertices<'a>(
        commands: impl Iterator<Item = &'a RenderCommand>,
        logical_width: f64,
        logical_height: f64,
    ) -> Vec<Vertex> {
        let filtered_commands: Vec<Vertex> = commands
            .filter_map(|c| match c {
                RenderCommand::Rectangle {
                    x,
                    y,
                    width,
                    height,
                } => {
                    let xf = screen_to_logical_device_coordinate(*x, logical_width);
                    let yf = screen_to_logical_device_coordinate(*y, logical_height);
                    let wf =
                        screen_to_logical_device_coordinate((x + *width as isize), logical_width);
                    let hf =
                        screen_to_logical_device_coordinate((y + *height as isize), logical_height);

                    let top_left = [xf, yf];
                    let top_right = [wf, yf];
                    let bottom_left = [xf, hf];
                    let bottom_right = [wf, hf];

                    Some(vec![
                        Vertex { position: top_left },
                        Vertex {
                            position: top_right,
                        },
                        Vertex {
                            position: bottom_left,
                        },
                        Vertex {
                            position: top_right,
                        },
                        Vertex {
                            position: bottom_right,
                        },
                        Vertex {
                            position: bottom_left,
                        },
                    ])
                }
                _ => None,
            })
            .flat_map(|vertices| vertices.into_iter())
            .collect();

        filtered_commands
    }
}

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r#"
            #version 450

            layout(location = 0) in vec2 position;
            layout(location = 0) out vec2 uv;
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                uv = position + vec2(0.5);
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
            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 0) uniform sampler2D tex;

            void main() {
                f_color = texture(tex, tex_coords);
            }
            "#
    }
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };
    dynamic_state.viewports = Some(vec![viewport]);

    images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}

fn screen_to_logical_device_coordinate(screen: isize, dimension: f64) -> f32 {
    return (screen as f32) / dimension as f32 * 2.0 - 1.0;
}

mod test {
    use super::screen_to_logical_device_coordinate;

    #[test]
    fn screen_to_logical_device_coordinate_handles_minimum() {
        assert_eq!(screen_to_logical_device_coordinate(0, 512.0), -1.0);
    }

    #[test]
    fn screen_to_logical_device_coordinate_handles_maximum() {
        assert_eq!(screen_to_logical_device_coordinate(512, 512.0), 1.0);
    }

    #[test]
    fn screen_to_logical_device_coordinate_handles_middle() {
        assert_eq!(screen_to_logical_device_coordinate(256, 512.0), 0.0);
    }
}
