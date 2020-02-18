use crate::backend::shape_pipeline::ShapeVertex;
use crate::{DrawCommand, DrawList, DrawingOptions, DrawingResult, TextureGreyScale, TextureRGBA};
use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Duration;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{
    AutoCommandBuffer, AutoCommandBufferBuilder, CommandBuffer, DynamicState,
};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract};
use vulkano::image::Dimensions::Dim2d;
use vulkano::image::{ImageViewAccess, StorageImage};
use vulkano::instance::{Instance, PhysicalDevice, QueueFamily};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::sync::GpuFuture;

pub(crate) struct Gpu {
    options: DrawingOptions,
    instance: Arc<Instance>,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    target_buffer: Arc<dyn ImageViewAccess + Send + Sync>,
    frame_buffer: Arc<dyn FramebufferAbstract + Send + Sync>,
    shape_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

pub(crate) struct GpuFrame {
    dynamic_state: DynamicState,
    command_buffer_builder: AutoCommandBufferBuilder,
    shape_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

pub(crate) struct SealedGpuFrame {
    commands: AutoCommandBuffer,
}

impl Gpu {
    pub fn initialize(options: DrawingOptions) -> DrawingResult<Gpu> {
        let instance = Instance::new(None, &vulkano_win::required_extensions(), None)
            .map_err(|_| "Failed to create Vulkan instance")?;
        let physical_device = PhysicalDevice::enumerate(&instance)
            .next()
            .ok_or("Failed to find a PhysicalDevice")?;
        let queue_family = physical_device
            .queue_families()
            .find(|&qf| qf.supports_graphics())
            .ok_or("Failed to find supported QueueFamily")?;

        let required_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let (device, mut queues) = Device::new(
            physical_device,
            physical_device.supported_features(),
            &required_extensions,
            [(queue_family, 0.5)].iter().cloned(),
        )
        .map_err(|_| "Failed to create a Vulkan Logical Device")?;

        let graphics_queue = queues
            .next()
            .ok_or("Did not receive a graphics queue with the Vulkan Logical Device")?;

        let format = vulkano::format::Format::R8G8B8A8Srgb;
        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    target: {
                        load: Clear,
                        store: Store,
                        format: format,
                        samples: 1,
                    }
                },
                pass: {
                    color: [target],
                    depth_stencil: {}
                }
            )
            .expect("Failed to create top level RenderPass"),
        );

        let target_buffer = StorageImage::new(
            device.clone(),
            Dim2d {
                width: options.width as u32,
                height: options.height as u32,
            },
            format,
            (&[graphics_queue.family()]).iter().cloned(),
        )
        .expect("Failed to create rasterization target buffer");

        let frame_buffer = Arc::new(
            Framebuffer::start(render_pass.clone())
                .add(target_buffer.clone())
                .expect("Failed to bind a target buffer to a frame buffer")
                .build()
                .expect("Failed to build a frame buffer"),
        );

        let shape_pipeline = shape_pipeline::create_pipeline(device.clone(), render_pass.clone());

        Ok(Gpu {
            options,
            instance,
            device,
            graphics_queue,
            render_pass,
            target_buffer,
            frame_buffer,
            shape_pipeline,
        })
    }

    pub fn begin_rasterizing(&self) -> GpuFrame {
        let viewports = Viewport {
            origin: [0.0, 0.0],
            dimensions: [self.options.width as f32, self.options.height as f32],
            depth_range: 0.0..1.0,
        };

        let dynamic_state = DynamicState {
            line_width: None,
            viewports: Some(vec![viewports]),
            reference: None,
            compare_mask: None,
            scissors: None,
            write_mask: None,
        };

        GpuFrame::new(
            self.device.clone(),
            self.graphics_queue.family(),
            self.frame_buffer.clone(),
            dynamic_state,
            self.shape_pipeline.clone(),
        )
    }

    pub fn submit_commands(&mut self, sealed_gpu_frame: SealedGpuFrame) {
        println!("foo");
        sealed_gpu_frame
            .commands
            .execute(self.graphics_queue.clone())
            .expect("Failed to execute command buffer")
            .then_signal_fence_and_flush()
            .expect("Failed to then_signal_fence_and_flush")
            .wait(Some(Duration::from_millis(5000)))
            .expect("Failed to wait");
    }

    pub fn load_texture_rgba(&self, texture: &TextureRGBA) -> DrawingResult<()> {
        unimplemented!()
    }

    pub fn load_texture_greyscale(&self, texture: &TextureGreyScale) -> DrawingResult<()> {
        unimplemented!()
    }
}

impl GpuFrame {
    pub fn new(
        device: Arc<Device>,
        queue_family: QueueFamily,
        target: Arc<dyn FramebufferAbstract + Send + Sync>,
        dynamic_state: DynamicState,
        shape_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> GpuFrame {
        let command_buffer_builder =
            AutoCommandBufferBuilder::primary_one_time_submit(device, queue_family)
                .expect("Failed to create command buffer")
                .begin_render_pass(target, false, vec![[0.0, 0.0, 1.0, 1.0].into()])
                .expect("Failed to begin render pass");

        GpuFrame {
            command_buffer_builder,
            dynamic_state,
            shape_pipeline,
        }
    }

    pub fn build_command_buffer(mut self, draw_list: &DrawList) -> DrawingResult<SealedGpuFrame> {
        let mut iterator = draw_list.commands.iter();

        loop {
            if let Some(head) = iterator.next() {
                match head {
                    DrawCommand::Shape { brush, .. } => {
                        let vertices: Vec<ShapeVertex> = vec![
                            ShapeVertex {
                                position: [-1.0, -1.0],
                                uv_input: [-1.0, -1.0],
                                kind_input: 1,
                            },
                            ShapeVertex {
                                position: [1.0, -1.0],
                                uv_input: [1.0, -1.0],
                                kind_input: 1,
                            },
                            ShapeVertex {
                                position: [-1.0, 1.0],
                                uv_input: [-1.0, 1.0],
                                kind_input: 1,
                            },
                            ShapeVertex {
                                position: [1.0, -1.0],
                                uv_input: [1.0, -1.0],
                                kind_input: 1,
                            },
                            ShapeVertex {
                                position: [1.0, 1.0],
                                uv_input: [1.0, 1.0],
                                kind_input: 1,
                            },
                            ShapeVertex {
                                position: [-1.0, 1.0],
                                uv_input: [-1.0, 1.0],
                                kind_input: 1,
                            },
                        ];

                        let vertex_buffer: Vec<Arc<dyn BufferAccess + Send + Sync + 'static>> = {
                            vec![CpuAccessibleBuffer::from_iter(
                                self.shape_pipeline.device().clone(),
                                BufferUsage::all(),
                                vertices.into_iter(),
                            )
                            .unwrap()]
                        };

                        self.command_buffer_builder = self
                            .command_buffer_builder
                            .draw(
                                self.shape_pipeline.clone(),
                                &self.dynamic_state,
                                vertex_buffer,
                                (),
                                (),
                            )
                            .expect("Failed to draw shapes");
                    }
                    _ => {
                        break;
                    }
                }
            } else {
                break;
            }
        }

        let command_buffer = self
            .command_buffer_builder
            .end_render_pass()
            .expect("Failed to end_render_pass")
            .build()
            .unwrap();

        Ok(SealedGpuFrame::new(command_buffer))
    }
}

impl SealedGpuFrame {
    pub fn new(commands: AutoCommandBuffer) -> Self {
        SealedGpuFrame { commands }
    }
}

mod shape_pipeline {
    use crate::DrawingResult;
    use std::sync::Arc;
    use vulkano::device::Device;
    use vulkano::framebuffer::{RenderPassAbstract, Subpass};
    use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};

    #[derive(Default, Debug, Clone)]
    pub struct ShapeVertex {
        pub position: [f32; 2],
        pub uv_input: [f32; 2],
        pub kind_input: i32,
    }
    vulkano::impl_vertex!(ShapeVertex, position, uv_input, kind_input);

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
            
            layout(location = 0) out vec2 uv_output;
            layout(location = 1) out int kind_out;
            
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                uv_output = uv_input;
                kind_out = kind_input;
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
            layout(location = 1) flat in int kind_in;
            
            layout(location = 0) out vec4 color_out;

            void main() {
                if (abs(uv_in.x) < 0.05 && abs(uv_in.y) < 0.05) {
                    color_out = vec4(1.0, 0.0, 0.0, 1.0);
                } else {
                    color_out = vec4(0.0, 1.0, 0.0, 1.0);
                }
            }
            "#
        }
    }
}
