use aabb::*;
use gfx::buffer::{Buffer, BufferUsage};
use gfx::context::Context;
use nalgebra::*;
use std::f32;

#[derive(Copy, Clone, Debug)]
pub struct Vertex3 {
    pub pos: Point3<f32>,
    pub normal: Vector3<f32>,
    pub tangent: Vector3<f32>,
    pub uv: Vector2<f32>,
}

#[derive(Debug)]
pub struct Mesh<V: Copy + 'static> {
    vbo: Buffer<[V]>,
    ibo: Option<Buffer<[i32]>>,
    vertex_count: usize,
    index_count: usize,
}

pub fn calculate_aabb(vertices: &[Vertex3]) -> AABB<f32> {
    let mut pmin = Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut pmax = Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

    for v in vertices {
        pmin.x = f32::min(pmin.x, v.pos.x);
        pmin.y = f32::min(pmin.y, v.pos.y);
        pmin.z = f32::min(pmin.z, v.pos.z);
        pmax.x = f32::max(pmax.x, v.pos.x);
        pmax.y = f32::max(pmax.y, v.pos.y);
        pmax.z = f32::max(pmax.z, v.pos.z);
    }

    AABB {
        min: pmin,
        max: pmax,
    }
}

impl<V: Copy + 'static> Mesh<V> {
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    pub fn index_count(&self) -> usize {
        self.index_count
    }

    pub fn new(context: &Context, vertices: &[V], indices: Option<&[i32]>) -> Mesh<V> {
        Mesh {
            vbo: Buffer::with_data(context, BufferUsage::DEFAULT, vertices),
            ibo: indices.map(|indices| Buffer::with_data(context, BufferUsage::DEFAULT, indices)),
            vertex_count: vertices.len(),
            index_count: indices.map(|indices| indices.len()).unwrap_or(0),
        }
    }

    pub fn vertex_buffer(&self) -> &Buffer<[V]> {
        &self.vbo
    }

    pub fn index_buffer(&self) -> Option<&Buffer<[i32]>> {
        self.ibo.as_ref()
    }
}
