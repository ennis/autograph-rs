use nalgebra::{Point3,Vector3,Vector2};
use gfx::{Buffer, BufferUsage, Context};
use std::rc::Rc;

#[derive(Copy,Clone,Debug)]
pub struct Vertex3
{
    pub pos: Point3<f32>,
    pub normal: Vector3<f32>,
    pub tangent: Vector3<f32>,
    pub uv: Vector2<f32>
}

#[derive(Debug)]
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

    pub fn new<T: Copy>(context: Rc<Context>, vertices: &[T], indices: Option<&[i32]>) -> Mesh {
        Mesh {
            vbo: Buffer::with_data(context.clone(), BufferUsage::DEFAULT, vertices),
            ibo: indices.map(|indices| Buffer::with_data(context.clone(), BufferUsage::DEFAULT, indices)),
            vertex_count: vertices.len(),
            index_count: indices.map(|indices| indices.len()).unwrap_or(0)
        }
    }
}
