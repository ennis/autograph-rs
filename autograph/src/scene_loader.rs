use assimp_sys::*;
use id_table::{ID, IDTable};
use scene_object::{SceneObject, SceneObjects, SceneMesh};
use aabb::AABB;
use mesh::{Mesh, Vertex3, calculate_aabb};
use nalgebra::*;
use itertools::Zip;
use gfx;
use std::rc::Rc;
use cache::{Cache, CacheTrait};
use std::slice;
use std::path::Path;
use std::ffi::{CString,CStr};

struct AssimpSceneImporter<'a>
{
    path: &'a Path,
    ids: &'a mut IDTable,
    cache: Rc<Cache>,
    ctx: Rc<gfx::Context>,
    scene_objects: &'a SceneObjects
}

unsafe fn import_mesh<'a>(importer: &AssimpSceneImporter<'a>, scene: *const AiScene, index: usize)
    -> Rc<SceneMesh>
{
    let mesh_name = format!("{:?}:mesh_{}", &importer.path, index);
    assert!(index < (*scene).num_meshes as usize);
    let aimesh = *((*scene).meshes.offset(index as isize));

    let cached_mesh = importer.cache.get_or(&mesh_name, || {
        debug!("Creating mesh {}", mesh_name);
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

        let aabb = calculate_aabb(&verts);
        debug!("Imported mesh AABB {:?}", aabb);

        Rc::new(SceneMesh {
            mesh: Mesh::new(&importer.ctx, &verts, Some(&indices)),
            aabb: calculate_aabb(&verts)
        })
    }).unwrap();

    cached_mesh
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
    /*let local_transform : Affine3<f32> = try_convert(Matrix4::new(
        tr.a1, tr.a2, tr.a3, tr.a4,
        tr.b1, tr.b2, tr.b3, tr.b4,
        tr.c1, tr.c2, tr.c3, tr.c4,
        tr.d1, tr.d2, tr.d3, tr.d4,
    )).unwrap();*/
    let local_transform = Affine3::<f32>::identity();

    let meshes = slice::from_raw_parts((*node).meshes, (*node).num_meshes as usize);

    // load children
    if meshes.len() == 1 {
        // one mesh attached to this node: import it and attach to the node
        let mesh = import_mesh(importer, scene, meshes[0] as usize);
        // build node
        importer.scene_objects.insert(SceneObject {
            id,
            parent_id,
            name,
            local_transform,
            world_transform: Affine3::identity(),
            world_bounds: mesh.aabb,
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
            mesh: None,
            children: Vec::new()
        });

        for (im,m) in meshes.iter().enumerate() {
            let child_id = importer.ids.create_id();
            let mesh = import_mesh(importer, scene, *m as usize);
            importer.scene_objects.insert(SceneObject {
                id: child_id,
                parent_id: Some(id),
                name: format!("(mesh {})", im),
                local_transform: Affine3::identity(),
                world_transform: Affine3::identity(),
                world_bounds: mesh.aabb,
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

pub fn load_scene_file(path: &Path, ids: &mut IDTable, context: &Rc<gfx::Context>, cache: &Rc<Cache>, scene_objects: &mut SceneObjects) -> Result<ID, String>
{
    let c_path = CString::new(path.to_str().unwrap()).unwrap();
    debug!("Import scene {:?}", c_path);
    //let postproc_flags = AIPROCESS_OPTIMIZE_MESHES | AIPROCESS_OPTIMIZE_GRAPH |
    //   AIPROCESS_TRIANGULATE | AIPROCESS_JOIN_IDENTICAL_VERTICES |
    //    AIPROCESS_CALC_TANGENT_SPACE | AIPROCESS_SORT_BY_PTYPE;
    let postproc_flags = AIPROCESS_TARGET_REALTIME_QUALITY;
    unsafe {
        let aiscene = aiImportFile(c_path.as_ptr(), postproc_flags);
        let log = CStr::from_ptr(aiGetErrorString()).to_str().unwrap();
        if aiscene.is_null() {
            error!("Importing scene failed");
            return Err(format!("Failed to import scene: {}", log));
        }
        let p_root_node = (*aiscene).root_node;

        let root_id = {
            let mut scene_importer_state = AssimpSceneImporter {
                path,
                cache: cache.clone(),
                ctx: context.clone(),
                scene_objects,
                ids
            };
            import_node(&mut scene_importer_state, aiscene, p_root_node, None)
        };
        // commit scene_objects
        scene_objects.commit_changes();
        Ok(root_id)
    }
}
