use assimp_sys::*;
use id_table::{ID, IDTable};
use scene_object::{SceneObject, SceneObjects};
use aabb::AABB;
use mesh::{Mesh, Vertex3};
use nalgebra::*;
use itertools::Zip;
use gfx;
use std::rc::Rc;
use rc_cache::{Cache, Cached};
use std::slice;

struct AssimpSceneImporter<'a>
{
    path: String,
    ids: &'a mut IDTable,
    cache: &'a Cache,
    ctx: Rc<gfx::Context>,
    scene_objects: &'a SceneObjects
}

unsafe fn import_mesh<'a>(importer: &AssimpSceneImporter<'a>, scene: *const AiScene, index: usize)
    -> (Cached<Mesh>, AABB<f32>)
{
    let mesh_name = format!("{}:mesh_{}", &importer.path, index);
    assert!(index < (*scene).num_meshes as usize);
    let aimesh = *((*scene).meshes.offset(index as isize));

    let cached_mesh = importer.cache.get_or(&importer.path, || {
        let vertices = slice::from_raw_parts((*aimesh).vertices, (*aimesh).num_vertices as usize);
        let normals = slice::from_raw_parts((*aimesh).normals, (*aimesh).num_vertices as usize);
        let tangents = slice::from_raw_parts((*aimesh).tangents, (*aimesh).num_vertices as usize);
        let texcoords0 = slice::from_raw_parts((*aimesh).vertices, (*aimesh).num_vertices as usize);
        let verts: Vec<Vertex3> = Zip::new((vertices, normals, tangents, texcoords0)).map(|(v,n,t,uv)|
            Vertex3 {
                pos: Point3::new(v.x, v.y, v.z),
                normal: Vector3::new(n.x, n.y, n.z),
                uv: Vector2::new(uv.x, uv.y),
                tangent: Vector3::new(t.x,t.y,t.z)
            }
        ).collect();

        let indices: Vec<i32> = slice::from_raw_parts((*aimesh).faces, (*aimesh).num_faces as usize).iter().flat_map(|f| {
            assert!((*f).num_indices == 3);
            let f = slice::from_raw_parts((*f).indices, (*f).num_indices as usize);
            vec![f[0] as i32,f[1] as i32, f[2] as i32]
        }).collect();

        Mesh::new(importer.ctx.clone(), verts.as_slice(), Some(indices.as_slice()))
    }).unwrap();

    (cached_mesh, unimplemented!())
}

// go full unsafe
unsafe fn import_node<'a>(importer: &mut AssimpSceneImporter<'a>, scene: *const AiScene, node: *const AiNode, parent_id: Option<ID>) -> ID
{
    // create entity
    let id = importer.ids.create_id();
    let name = (*node).name.as_ref().to_owned();
    debug!("Importing node {}", name);
    // load transform
    let tr = (*node).transformation;
    // convert to nalgebra type
    let local_transform : Affine3<f32> = try_convert(Matrix4::new(
        tr.a1, tr.a2, tr.a3, tr.a4,
        tr.b1, tr.b2, tr.b3, tr.b4,
        tr.c1, tr.c2, tr.c3, tr.c4,
        tr.d1, tr.d2, tr.d3, tr.d4,
    )).unwrap();

    let meshes = slice::from_raw_parts((*node).meshes, (*node).num_meshes as usize);

    // load children
    if meshes.len() == 1 {
        // one mesh attached to this node: import it and attach to the node
        let (mesh, bounds) = import_mesh(importer, scene, meshes[0] as usize);
        // build node
        importer.scene_objects.insert(SceneObject {
            id,
            parent_id,
            name,
            local_transform,
            world_transform: Affine3::identity(),
            world_bounds: bounds,
            mesh_bounds: bounds,
            mesh: Some(mesh),
            children: Vec::new()
        });
    } else {
        // more than one mesh: import meshes in child nodes
        // create parent node
        importer.scene_objects.insert(SceneObject {
            id,
            parent_id,
            name,
            local_transform,
            world_transform: Affine3::identity(),
            world_bounds: AABB::empty(),
            mesh_bounds: AABB::empty(),
            mesh: None,
            children: Vec::new()
        });

        for (im,m) in meshes.iter().enumerate() {
            let child_id = importer.ids.create_id();
            let (mesh, bounds) = import_mesh(importer, scene, *m as usize);
            importer.scene_objects.insert(SceneObject {
                id: child_id,
                parent_id: Some(id),
                name: format!("(mesh {})", im),
                local_transform: Affine3::identity(),
                world_transform: Affine3::identity(),
                world_bounds: bounds,
                mesh_bounds: bounds,
                mesh: Some(mesh),
                children: Vec::new()
            });
        }
    }

    // import child nodes
    let children = slice::from_raw_parts((*node).children, (*node).num_children as usize);
    for child_node in children {
        import_node(importer, scene, *child_node, Some(id));
    }

    id
}

