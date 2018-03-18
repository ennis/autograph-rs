use super::Frame;
use gfx::{Framebuffer,GraphicsPipeline,DrawCmd,DrawCmdBuilder,DrawExt};

pub trait DrawUtilsExt<'queue>
{
    fn draw_quad<'frame, 'pipeline>(&'frame self, target: &Framebuffer, pipeline: &'pipeline GraphicsPipeline, ltrb: (f32, f32, f32, f32)) -> DrawCmdBuilder<'frame, 'queue, 'pipeline> where 'queue:'frame;
}

impl<'queue> DrawUtilsExt<'queue> for Frame<'queue>
{
    fn draw_quad<'frame, 'pipeline>(&'frame self, target: &Framebuffer, pipeline: &'pipeline GraphicsPipeline, ltrb: (f32, f32, f32, f32)) -> DrawCmdBuilder<'frame, 'queue, 'pipeline> where 'queue:'frame
    {
        let (left,right,top,bottom) = ltrb;
        let vertices = [
            [left,top],     [0.0f32,0.0f32],
            [right,top],    [1.0f32,0.0f32],
            [left,bottom],  [0.0f32,1.0f32],
            [left,bottom],  [0.0f32,1.0f32],
            [right,top],    [1.0f32,0.0f32],
            [right,bottom], [1.0f32,1.0f32],
        ];
        let vertices_gpu = self.upload(&vertices);
        self.draw(target, pipeline, DrawCmd::DrawArrays { first: 0, count: 6 }).with_vertex_buffer(0, &vertices_gpu)
    }
}