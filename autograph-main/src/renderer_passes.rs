use autograph::gfx;
use autograph::scene_object::SceneObjects;
use autograph::camera::Camera;
use nalgebra::*;
use std::sync::Arc;

gfx_pass! {
pass GBufferSetup(frame: &'pass gfx::Frame, width: u32, height: u32)
{
	read {
	}
	write {
	}
	create {
		#[framebuffer(fbo,0)]	// this annotation will create a framebuffer named fbo and bind the texture to the
		texture2D diffuse { 	// can be texture, or any of texture1d, texture2d, texture3d...
			format: R16G16B16A16_SFLOAT,	// any enumerator of gfx::TextureFormat
			width,		// can be an expression, can reference metadata of read and write inputs, can reference pass parameters
							// shortcut expressions are supported with pass parameters
			height,
			.. Default::default()
		},
		#[framebuffer(fbo,1)]
		texture2D normals {
			format: R16G16B16A16_SFLOAT,
			width,
			height,
			.. Default::default()
		},
		#[framebuffer(fbo,2)]
		texture2D material_id {
			format: R16G16B16A16_SFLOAT,
			width,
			height,
			.. Default::default()
		},
		#[framebuffer(fbo,depth)]
		texture2D depth {
			format: D32_SFLOAT,
			width,
			height,
			.. Default::default()
		}
	}

	// (optional) pipeline section
	// will load pipelines from a file, and make them available in the scope of execute()
	// internally, it uses a lazy_static block
	pipeline dummy {
		path: "data/shaders/deferred.glsl",
		.. Default::default()
	}

	// Validation of inputs (read & write)
	validate {
		assert(self.normals.width != 0);
		assert(self.normals.height != 0);
	}

	execute {
	    frame.clear_texture(diffuse, 0, &[0.125, 0.125, 0.48, 1.0]);
	    frame.clear_texture(normals, 0, &[0.0, 0.0, 0.0, 1.0]);
	    frame.clear_texture(material_id, 0, &[0.0, 0.0, 0.0, 1.0]);
	    frame.clear_depth_texture(depth, 0, 1.0);
	}
}
}

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

#[derive(Copy, Clone, Debug)]
pub enum DeferredDebugBuffer {
    Diffuse,
    Normals,
    MaterialID,
    Depth,
}

gfx_pass!{
pass DeferredDebug(frame: &'pass gfx::Frame, target: &'pass Arc<gfx::Framebuffer>, debug: DeferredDebugBuffer)
{
    read {
        texture2D diffuse {},
        texture2D normals {},
        texture2D material_id {},
        texture2D depth {}
    }
    write {
    }
    execute
    {
        frame.clear_framebuffer_color(target, 0, &[0.125, 0.125, 0.48, 1.0]);
        println!("Execute DeferredDebug!");
    }
}

}

/*gfx_pass!{
pass Present(frame: &'pass gfx::Frame, target: &'pass Arc<gfx::Framebuffer>)
{
    read {
        texture2D source {},
    }
    write {
    }
    pipeline BLIT_2D {
        path: "data/shaders/blit.glsl"
    }
    execute
    {

    }
}

}*/
