use autograph::gfx;
use autograph::scene_object::SceneObjects;

gfx_pass! {
pass GBufferSetup(width: u32, height: u32 /* pass parameters */)
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
	pipeline pipeline_name {
		file: "data/shaders/dummy.glsl",
		// rasterizer state, etc.
		// dynamic parameters?
		// preprocessor macros?
	}

	// Validation of inputs (read & write)
	validate {
		assert(self.normals.width != 0);
		assert(self.normals.height != 0);
	}

	execute {
		// some rust code (in particular, gfx commands)
	}
}
}

gfx_pass!{
pass RenderScene(scene: &SceneObjects)
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
    execute
    {
        println!("Execute RenderScene!");
    }
}
}

#[derive(Copy,Clone,Debug)]
pub enum DeferredDebugBuffer
{
    Diffuse,
    Normals,
    MaterialID,
    Depth,
}

gfx_pass!{
pass DeferredDebug(debug: DeferredDebugBuffer)
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
        println!("Execute DeferredDebug!");
    }
}

}


