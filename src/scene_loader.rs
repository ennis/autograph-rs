use assimp as ai;
use id_table::{ID, IDTable};
use scene_object::{SceneObject, SceneObjects};
use aabb::AABB;
use mesh::{Mesh, Vertex3};
use nalgebra::*;
use itertools::Zip;
use gfx;
use std::rc::Rc;
use rc_cache::{Cache, Cached};

struct AssimpSceneImporter<'a, 'cache>
{
    path: String,
    ids: &'a IDTable,
    cache: &'cache Cache,
    ctx: Rc<gfx::Context>,
}

fn import_mesh<'a, 'imp, 'cache: 'imp>(importer: &'imp AssimpSceneImporter<'a, 'cache>, scene: &ai::Scene, index: usize)
    -> (Cached<Mesh>, AABB<f32>)
{
    let mesh_name = format!("{}:mesh_{}", &importer.path, index);
    let aimesh = scene.mesh(index).unwrap();

    let cached_mesh = importer.cache.get_or(&importer.path, || {
        let verts: Vec<Vertex3> = Zip::new((aimesh.vertex_iter(), aimesh.normal_iter(), aimesh.tangent_iter(), aimesh.texture_coords_iter(0))).map(|(v,n,t,uv)|
            Vertex3 {
                pos: Point3::new(v.x, v.y, v.z),
                normal: Vector3::new(n.x, n.y, n.z),
                uv: Vector2::new(uv.x, uv.y),
                tangent: Vector3::new(t.x,t.y,t.z)
            }
        ).collect();

        let indices: Vec<i32> = aimesh.face_iter().flat_map(|f| {
            vec![f[0] as i32,f[1] as i32, f[2] as i32]
        }).collect();

        Mesh::new(importer.ctx, verts.as_slice(), Some(indices.as_slice()))
    }).unwrap();

    (cached_mesh, unimplemented!())
}