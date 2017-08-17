//! The deferred renderer module
//!
//!
use autograph::gfx;
use autograph::scene_object::SceneObjects;
use autograph::camera::Camera;
use nalgebra::*;
use std::sync::Arc;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CameraParameters {
    view_matrix: Matrix4<f32>,
    proj_matrix: Matrix4<f32>,
    viewproj_matrix: Matrix4<f32>,
    inverse_proj_matrix: Matrix4<f32>,
    prev_viewproj_matrix_velocity: Matrix4<f32>,
    viewproj_matrix_velocity: Matrix4<f32>,
    temporal_aa_offset: [f32; 2],
}

impl CameraParameters {
    pub fn from_camera(cam: &Camera, temporal_aa_offset: (f32, f32)) -> CameraParameters {
        let view_matrix = cam.view.to_homogeneous();
        let proj_matrix = cam.projection.unwrap();
        let viewproj_matrix = proj_matrix * view_matrix;
        let inverse_proj_matrix = cam.projection.inverse();

        CameraParameters {
            view_matrix,
            proj_matrix,
            viewproj_matrix,
            inverse_proj_matrix,
            viewproj_matrix_velocity: viewproj_matrix,
            prev_viewproj_matrix_velocity: viewproj_matrix,
            temporal_aa_offset: [0.0; 2], // TODO
        }
    }
}

// Per-object parameters
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct ObjectParameters {
    model_matrix: Matrix4<f32>,
    prev_model_matrix: Matrix4<f32>,
    object_id: i32,
}

fn do_render_scene(
    scene_objects: &SceneObjects,
    camera: &Camera,
    frame: &gfx::Frame,
    target: &Arc<gfx::Framebuffer>,
    pipeline: &Arc<gfx::GraphicsPipeline>)
{
    use autograph::gfx::AsSlice;

    // create camera buffer to send to GPU
    let cam_buffer = frame.upload(camera);

    for (id,obj) in scene_objects.iter() {
        // build draw command!
        let obj = obj.borrow();

        if let Some(ref sm) = obj.mesh {
            //debug!("Render id {:?}", id);
            let objparams = frame.upload(&ObjectParameters {
                model_matrix: obj.world_transform.to_homogeneous(),
                prev_model_matrix: obj.world_transform.to_homogeneous(),
                object_id: id.idx as i32
            });

            frame.begin_draw(target, pipeline)
                .with_vertex_buffer(0, &sm.mesh.vertex_buffer().as_slice())
                .with_index_buffer(&sm.mesh.index_buffer().unwrap().as_slice())
                .with_uniform_buffer(0, &cam_buffer)
                .with_uniform_buffer(1, &objparams)
                .draw_indexed(0, sm.mesh.index_count(), 0);
        }
    }
}

gfx_pass!{
pass RenderScene(scene_objects: &'pass SceneObjects, camera: &'pass Camera, frame: &'pass gfx::Frame)
{
    read {}
    write {
		#[framebuffer(fbo,0)]
        texture2D diffuse {},
		#[framebuffer(fbo,1)]
        texture2D normals {},
		#[framebuffer(fbo,2)]
        texture2D material_id {},
		#[framebuffer(fbo,depth)]
        texture2D depth {}
    }

    pipeline DEFERRED {
        path: "data/shaders/deferred.glsl",
        .. Default::default()
    }

    execute
    {
        // make a framebuffer on-the-fly
        // TODO cache it: in FrameGraphAllocator
        let target = Arc::new(gfx::FramebufferBuilder::new(frame.queue().context())
            .attach_texture(0, diffuse)
            .attach_texture(1, normals)
            .attach_texture(2, material_id)
            .attach_depth_texture(depth)
            .build());

        do_render_scene(
            scene_objects,
            camera,
            frame,
            &target,
            &DEFERRED);
    }
}
}

pub use self::RenderScene::*;