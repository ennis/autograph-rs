use imgui;
use imgui_sys;
use autograph::gfx;
use autograph::cache::{Cache, CacheTrait};
use autograph::gl;
use autograph::gl::types::*;
use autograph::gfx::glsl::GraphicsPipelineBuilderExt;
use autograph::gfx::draw::{DrawExt, DrawCmd};
use glutin;
use std::path::Path;
use std::sync::Arc;
use failure::Error;

pub struct Renderer {
    pipeline: gfx::GraphicsPipeline,
    texture: gfx::RawTexture,
}

static IMGUI_SHADER_PATH: &str = "data/shaders/imgui.glsl";

fn load_pipeline(gctx: &gfx::Context, path: &Path) -> Result<gfx::GraphicsPipeline, Error> {
    Ok(gfx::GraphicsPipelineBuilder::new()
        .with_glsl_file(path)?
        .with_rasterizer_state(&gfx::RasterizerState {
            fill_mode: gl::FILL,
            ..Default::default()
        })
        .with_all_blend_states(&gfx::BlendState {
            enabled: true,
            mode_rgb: gl::FUNC_ADD,
            mode_alpha: gl::FUNC_ADD,
            func_src_rgb: gl::SRC_ALPHA,
            func_dst_rgb: gl::ONE_MINUS_SRC_ALPHA,
            func_src_alpha: gl::ONE,
            func_dst_alpha: gl::ZERO,
        })
        .build(gctx)?)
}

impl Renderer {
    pub fn new(
        imgui: &mut imgui::ImGui,
        gctx: &gfx::Context,
        cache: &Arc<Cache>,
    ) -> Renderer {
        let pipeline = cache
            .add_and_watch(IMGUI_SHADER_PATH.to_owned(), |path, reload_reason| {
                load_pipeline(gctx, Path::new(path)).ok()
            })
            .unwrap();

        let texture = imgui.prepare_texture(|handle| {
            let desc = gfx::TextureDesc {
                format: gfx::Format::R8G8B8A8_SRGB,
                dimensions: gfx::TextureDimensions::Tex2D,
                options: gfx::TextureOptions::empty(),
                width: handle.width,
                height: handle.height,
                depth: 1,
                mip_map_count: gfx::MipMaps::Count(1),
                sample_count: 1,
            };
            let texture = gfx::RawTexture::with_pixels(gctx, &desc, handle.pixels);
            texture
        });
        imgui.set_texture_id(texture.gl_object() as usize);

        Renderer { pipeline, texture }
    }

    pub fn render<'a>(
        &mut self,
        frame: &gfx::Frame,
        target: &gfx::Framebuffer,
        ui: imgui::Ui<'a>,
    ) {
        // hot-reload pipeline from file
        //self.pipeline.update();
        ui.render(move |ui, draw_list| -> Result<(), String> {
            self.render_draw_list(frame, target,  ui, &draw_list)
        });
    }

    pub fn render_draw_list<'a>(
        &mut self,
        frame: &gfx::Frame,
        target: &gfx::Framebuffer,
        ui: &imgui::Ui<'a>,
        draw_list: &imgui::DrawList<'a>,
    ) -> Result<(), String>
    {
        let vertex_buffer = frame.upload(draw_list.vtx_buffer);
        let index_buffer = frame.upload(draw_list.idx_buffer);
        let (width, height) = ui.imgui().display_size();
        let (scale_width, scale_height) = ui.imgui().display_framebuffer_scale();

        if width == 0.0 || height == 0.0 {
            return Ok(());
        }

        let matrix: [[f32; 4]; 4] = [
            [2.0 / width as f32, 0.0, 0.0, 0.0],
            [0.0, 2.0 / -(height as f32), 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        let font_texture_id = self.texture.gl_object() as usize;
        let mut idx_start = 0 as usize;

        for cmd in draw_list.cmd_buffer {
            // We don't support custom textures...yet!
            assert!(cmd.texture_id as usize == font_texture_id);

            let idx_end = idx_start + cmd.elem_count as usize;

            let uniforms = frame.upload(&matrix);

            frame.draw(target, &self.pipeline, DrawCmd::DrawIndexed { first: idx_start, count: cmd.elem_count as usize, base_vertex: 0 })
                .with_vertex_buffer(0, &vertex_buffer)
                .with_index_buffer(&index_buffer)
                .with_uniform_buffer(0, &uniforms)
                .with_texture(0,
                              &self.texture,
                              &gfx::SamplerDesc {
                                  addr_u: gfx::TextureAddressMode::Wrap,
                                  addr_v: gfx::TextureAddressMode::Wrap,
                                  addr_w: gfx::TextureAddressMode::Wrap,
                                  mag_filter: gfx::TextureMagFilter::Nearest,
                                  min_filter: gfx::TextureMinFilter::Linear,
                              });

            /*frame.begin_draw(target, &self.pipeline)
                .with_vertex_buffer(0, &vertex_buffer)
                .with_index_buffer(&index_buffer)
                .with_uniform_buffer(0, &uniforms)
                .with_texture(
                    0,
                    &self.texture,
                    &gfx::SamplerDesc {
                        addr_u: gfx::TextureAddressMode::Wrap,
                        addr_v: gfx::TextureAddressMode::Wrap,
                        addr_w: gfx::TextureAddressMode::Wrap,
                        mag_filter: gfx::TextureMagFilter::Nearest,
                        min_filter: gfx::TextureMinFilter::Linear,
                    },
                )
                .with_all_scissors(Some((
                    (cmd.clip_rect.x * scale_width) as i32,
                    ((height - cmd.clip_rect.w) * scale_height) as i32,
                    ((cmd.clip_rect.z - cmd.clip_rect.x) * scale_width) as i32,
                    ((cmd.clip_rect.w - cmd.clip_rect.y) * scale_height) as i32,
                )))
                .draw_indexed(
                    idx_start,
                    cmd.elem_count as usize,
                    0,
                );*/

            idx_start = idx_end;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}


pub fn init(
    context: &gfx::Context,
    cache: &Arc<Cache>,
    replacement_font: Option<&str>,
) -> (imgui::ImGui, Renderer, MouseState) {
    // setup ImGui
    let mut imgui = imgui::ImGui::init();
    // load font from file
    if let Some(replacement_font) = replacement_font {
        unsafe {
            use std::ffi::{CStr, CString};
            let path = CString::new(replacement_font).unwrap();
            let imgui_io = &mut *imgui_sys::igGetIO();
            imgui_sys::ImFontAtlas_AddFontFromFileTTF(
                imgui_io.fonts,
                path.as_ptr(),
                20.0,
                0 as *const _,
                0 as *const _,
            );
        };
    }

    // create an imgui renderer
    let renderer = Renderer::new(&mut imgui, context, cache);
    (imgui, renderer, MouseState::default())
}

pub fn handle_event(
    imgui: &mut imgui::ImGui,
    event: &glutin::Event,
    mouse_state: &mut MouseState,
) -> bool {
    use glutin::{ElementState, Event, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent};
    use glutin::WindowEvent::*;

    match event {
        &Event::WindowEvent { ref event, .. } => {
            match event {
                &KeyboardInput { input, .. } => {
                    use glutin::VirtualKeyCode as Key;
                    let pressed = input.state == ElementState::Pressed;
                    match input.virtual_keycode {
                        Some(Key::Tab) => imgui.set_key(0, pressed),
                        Some(Key::Left) => imgui.set_key(1, pressed),
                        Some(Key::Right) => imgui.set_key(2, pressed),
                        Some(Key::Up) => imgui.set_key(3, pressed),
                        Some(Key::Down) => imgui.set_key(4, pressed),
                        Some(Key::PageUp) => imgui.set_key(5, pressed),
                        Some(Key::PageDown) => imgui.set_key(6, pressed),
                        Some(Key::Home) => imgui.set_key(7, pressed),
                        Some(Key::End) => imgui.set_key(8, pressed),
                        Some(Key::Delete) => imgui.set_key(9, pressed),
                        Some(Key::Back) => imgui.set_key(10, pressed),
                        Some(Key::Return) => imgui.set_key(11, pressed),
                        Some(Key::Escape) => imgui.set_key(12, pressed),
                        Some(Key::A) => imgui.set_key(13, pressed),
                        Some(Key::C) => imgui.set_key(14, pressed),
                        Some(Key::V) => imgui.set_key(15, pressed),
                        Some(Key::X) => imgui.set_key(16, pressed),
                        Some(Key::Y) => imgui.set_key(17, pressed),
                        Some(Key::Z) => imgui.set_key(18, pressed),
                        Some(Key::LControl) | Some(Key::RControl) => imgui.set_key_ctrl(pressed),
                        Some(Key::LShift) | Some(Key::RShift) => imgui.set_key_shift(pressed),
                        Some(Key::LAlt) | Some(Key::RAlt) => imgui.set_key_alt(pressed),
                        Some(Key::LWin) | Some(Key::RWin) => imgui.set_key_super(pressed),
                        _ => {}
                    }
                }
                &CursorMoved {
                    position: (x, y), ..
                } => mouse_state.pos = (x as i32, y as i32),
                &MouseInput { state, button, .. } => match button {
                    MouseButton::Left => mouse_state.pressed.0 = state == ElementState::Pressed,
                    MouseButton::Right => mouse_state.pressed.1 = state == ElementState::Pressed,
                    MouseButton::Middle => mouse_state.pressed.2 = state == ElementState::Pressed,
                    _ => {}
                },
                &MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_, y),
                    phase: TouchPhase::Moved,
                    ..
                } |
                &MouseWheel {
                    delta: MouseScrollDelta::PixelDelta(_, y),
                    phase: TouchPhase::Moved,
                    ..
                } => mouse_state.wheel = y,
                &ReceivedCharacter(c) => imgui.add_input_character(c),
                _ => (),
            }

            // update mouse

            let scale = imgui.display_framebuffer_scale();
            imgui.set_mouse_pos(
                mouse_state.pos.0 as f32 / scale.0,
                mouse_state.pos.1 as f32 / scale.1,
            );
            imgui.set_mouse_down(&[
                mouse_state.pressed.0,
                mouse_state.pressed.1,
                mouse_state.pressed.2,
                false,
                false,
            ]);
            imgui.set_mouse_wheel(mouse_state.wheel / scale.1);
            mouse_state.wheel = 0.0;
            true
        }
        _ => false,
    }
}
