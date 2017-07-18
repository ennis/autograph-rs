use nalgebra::{Point3,Vector3,Vector2};
//use gfx::{Buffer, Context};

pub struct Vertex3
{
    pub pos: Point3<f32>,
    pub normal: Vector3<f32>,
    pub tangent: Vector3<f32>,
    pub uv: Vector2<f32>
}

pub struct Mesh<'ctx>
{
    vbo: Buffer<'ctx>,
    ibo: Option<Buffer<'ctx>>,
    vertex_count: usize,
    index_count: usize
}

impl<'ctx> Mesh<'ctx>
{
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    pub fn index_count(&self) -> usize {
        self.index_count
    }

    pub fn new<'a, T>(context: &'a Context, vertices: &[T], indices: Option<&[i32]>) -> Mesh<'a> {
        //Mesh {
        //    vbo: Buffer::new(context, vertices.o)
        //}
        unimplemented!()
    }
}