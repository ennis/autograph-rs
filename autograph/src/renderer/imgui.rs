use imgui;
use gfx;
use rc_cache::{Cache, CacheTrait};
use std::rc::Rc;
use shader_compiler::*;
use std::path::Path;
use gl;
use gl::types::*;

pub struct Renderer
{
    pipeline: Rc<gfx::GraphicsPipeline>,
    texture: Rc<gfx::Texture>
}

static IMGUI_SHADER_PATH: &str = "data/shaders/imgui.glsl";

fn load_pipeline(ctx: Rc<gfx::Context>, path: &Path) -> Result<Rc<gfx::GraphicsPipeline>, String>
{
    let compiled_shaders = compile_shaders_from_combined_source(path)?;
    Ok(Rc::new(
        gfx::GraphicsPipelineBuilder::new()
                .with_vertex_shader(compiled_shaders.vertex)
                .with_fragment_shader(compiled_shaders.fragment)
                .with_geometry_shader(compiled_shaders.geometry)
                .with_tess_eval_shader(compiled_shaders.tess_eval)
                .with_tess_control_shader(compiled_shaders.tess_control)
                .with_primitive_topology(compiled_shaders.primitive_topology)
                .with_rasterizer_state(&gfx::RasterizerState{
                    fill_mode: gl::FILL,
                    .. Default::default()
                })
                .with_all_blend_states(&gfx::BlendState {
                    enabled: true,
                    mode_rgb: gl::FUNC_ADD,
                    mode_alpha: gl::FUNC_ADD,
                    func_src_rgb: gl::SRC_ALPHA,
                    func_dst_rgb: gl::ONE_MINUS_SRC_ALPHA,
                    func_src_alpha: gl::ONE,
                    func_dst_alpha: gl::ZERO
                })
                .with_input_layout(&compiled_shaders.input_layout)
                .build(ctx.clone()).map_err(|gfx::GraphicsPipelineBuildError::ProgramLinkError(log)| format!("Program link error: {}", log))?
        ))
}

impl Renderer
{
    pub fn new(imgui: &mut imgui::ImGui, context: Rc<gfx::Context>, cache: Rc<Cache>) -> Renderer
    {
        let pipeline = cache.add_and_watch(IMGUI_SHADER_PATH.to_owned(), |path, reload_reason| {
            load_pipeline(context.clone(), Path::new(path)).ok()
        }).unwrap();

        let texture = imgui.prepare_texture(|handle| {
            let desc = gfx::TextureDesc {
                format: gfx::TextureFormat::R8G8B8A8_SRGB,
                dimensions: gfx::TextureDimensions::Tex2D,
                options: gfx::TextureOptions::empty(),
                width: handle.width,
                height: handle.height,
                depth: 1,
                mip_map_count: gfx::MipMaps::Count(1),
                sample_count: 1
            };
            let mut texture = gfx::Texture::new(context, &desc );
            texture.upload_region(0, (0,0,0), (handle.width,handle.height,1), handle.pixels);
            Rc::new(texture)
        });
        imgui.set_texture_id(texture.object() as usize);

        Renderer {
            pipeline,
            texture
        }
    }

    pub fn render<'a>(&mut self, frame: &gfx::Frame, target: Rc<gfx::Framebuffer>, upload_buf: &gfx::UploadBuffer, ui: imgui::Ui<'a>)
    {
        // hot-reload pipeline from file
        //self.pipeline.update();

        ui.render(move |ui, draw_list| -> Result<(),String> {
            self.render_draw_list(frame, target.clone(), upload_buf, ui, &draw_list);
            unimplemented!()
        });

    }

    pub fn render_draw_list<'a>(&mut self, frame: &gfx::Frame, target: Rc<gfx::Framebuffer>, upload_buf: &gfx::UploadBuffer, ui: &imgui::Ui<'a>, draw_list: &imgui::DrawList<'a>) -> Result<(),String> {

        let vertex_buffer = upload_buf.upload(frame, draw_list.vtx_buffer, 64);
        let index_buffer = upload_buf.upload(frame, draw_list.idx_buffer, 64);
        let (width,height) = ui.imgui().display_size();
        let (scale_width, scale_height) = ui.imgui().display_framebuffer_scale();

        if width == 0.0 || height == 0.0 {
            return Ok(());
        }

        let matrix =
            [[2.0 / width as f32, 0.0, 0.0, 0.0],
             [0.0, 2.0 / -(height as f32), 0.0, 0.0],
             [0.0, 0.0, -1.0, 0.0],
             [-1.0, 1.0, 0.0, 1.0]];

        let font_texture_id = self.texture.object() as usize;
        let mut idx_start = 0 as usize;

        for cmd in draw_list.cmd_buffer {
            // We don't support custom textures...yet!
            assert!(cmd.texture_id as usize == font_texture_id);

            let idx_end = idx_start + cmd.elem_count as usize;

            gfx::DrawCommandBuilder::new(frame, target.clone(), self.pipeline.clone())
                    .with_vertex_buffer(0, &vertex_buffer)
                    .with_index_buffer(&index_buffer)
                    .with_uniform_buffer(0, &upload_buf.upload(frame, &matrix, 256))
                    .with_all_scissors(Some (
                        (
                            (cmd.clip_rect.x * scale_width) as i32,
                            ((height - cmd.clip_rect.w) * scale_height) as i32,
                            ((cmd.clip_rect.z - cmd.clip_rect.x) * scale_width) as i32,
                            ((cmd.clip_rect.w - cmd.clip_rect.y) * scale_height) as i32
                        )
                    ))
                    .command(&gfx::DrawIndexed {
                        first: idx_start,
                        count: cmd.elem_count as usize,
                        base_vertex: 0
                    });

            idx_start = idx_end;
        }

        Ok(())
    }

}
