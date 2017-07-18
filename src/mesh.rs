use nalgebra::{Point3,Vector3,Vector2};
use gfx::{Buffer, Context};
use std::rc::Rc;

pub struct Vertex3
{
    pub pos: Point3<f32>,
    pub normal: Vector3<f32>,
    pub tangent: Vector3<f32>,
    pub uv: Vector2<f32>
}

pub struct Mesh
{
    vbo: Buffer,
    ibo: Option<Buffer>,
    vertex_count: usize,
    index_count: usize
}

impl Mesh
{
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    pub fn index_count(&self) -> usize {
        self.index_count
    }

    pub fn new<T>(context: Rc<Context>, vertices: &[T], indices: Option<&[i32]>) -> Mesh {
        //Mesh {
        //    vbo: Buffer::new(context, vertices.o)
        //}
        unimplemented!()
    }
}