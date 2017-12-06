//! Defines the ShaderInterface trait
//! The trait should be implemented by a procedural macro

trait ShaderInterface {
    // Bind the shader parameters to the OpenGL pipeline
    unsafe fn bind_gl(&self);
}
