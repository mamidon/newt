use crate::newt_render::RenderCommand;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};

pub mod boxes;
pub mod glyphs;

pub struct CommandBufferWritingInfo<'a, I>
where
    I: Iterator<Item = &'a RenderCommand>,
{
    commands: I,
    image_index: usize,
    logical_width: f64,
    logical_height: f64,
}

impl<'a, I> CommandBufferWritingInfo<'a, I>
where
    I: Iterator<Item = &'a RenderCommand> + Clone,
{
    pub fn initialize(
        commands: I,
        image_index: usize,
        logical_width: f64,
        logical_height: f64,
    ) -> CommandBufferWritingInfo<'a, I> {
        CommandBufferWritingInfo {
            commands,
            image_index,
            logical_width,
            logical_height,
        }
    }

    pub fn create_attachments<S, O>(&self, selector: &S) -> Vec<O>
    where
        S: Fn(&Self) -> Vec<O>,
    {
        selector(self)
    }
}
