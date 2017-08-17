//! Deferred setup pass: creates the G-buffers
//!
//!
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
		#[framebuffer(fbo,0)]
		texture2D diffuse {
			format: R16G16B16A16_SFLOAT,
			width,
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

pub use self::GBufferSetup::*;
