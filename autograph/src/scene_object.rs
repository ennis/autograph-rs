use nalgebra::{Affine3};
use id_table::ID;
use std::rc::Rc;
use aabb::AABB;
use std::collections::HashMap;
use std::cell::RefCell;
use mesh::Mesh;
use rc_cache::Cached;

///
///
///
///
pub struct SceneObject
{
    pub id: ID,
    pub name: String,
    pub parent_id: Option<ID>,                  // TODO Option<ID> ?
    pub local_transform: Affine3<f32>,
    pub world_transform: Affine3<f32>,
    pub world_bounds: AABB<f32>,        // TODO Should be Option<AABB<f32>>, since an object may not have world bounds
    pub mesh_bounds: AABB<f32>,         // Same, as an object may not have an attached mesh
    pub children: Vec<ID>,
    pub mesh: Option<Cached<Mesh>>      // TODO should this be in its own component map?
}

impl SceneObject
{
}

enum SceneGraphChange
{
    Parent(ID,ID),
    Orphan(ID,ID),
    Insert(SceneObject),
    Remove(ID)
}

pub struct SceneObjects
{
    scene_objects: HashMap<ID, RefCell<SceneObject>>,
    changes: RefCell<Vec<SceneGraphChange>>
}

/// The actual scene graph is changed at the end of the frame
/// This way the pointers to the scene objects stay stable within a frame
impl SceneObjects
{
    pub fn new() -> SceneObjects {
        SceneObjects {
            scene_objects: HashMap::new(),
            changes: RefCell::new(Vec::new())
        }
    }

    /// Add a parent/child relationship between the two IDs
    pub fn parent(&self, parent: ID, child: ID)
    {
        self.changes.borrow_mut().push(SceneGraphChange::Parent(parent, child));
    }

    pub fn remove(&self, id: ID)
    {
        self.changes.borrow_mut().push(SceneGraphChange::Remove(id));
    }

    pub fn orphan(&self, parent: ID, child: ID)
    {
        self.changes.borrow_mut().push(SceneGraphChange::Orphan(parent, child));
    }

    pub fn insert(&self, scene_object: SceneObject)
    {
        self.changes.borrow_mut().push(SceneGraphChange::Insert(scene_object));
    }

    fn calculate_transforms_rec(&self, ids: &[ID], parent_transform: &Affine3<f32>)
    {
        for id in ids.iter() {
            let mut so = self.scene_objects.get(&id).unwrap().borrow_mut();  // borrow mut self
            so.world_transform = parent_transform * so.local_transform;
            self.calculate_transforms_rec(so.children.as_slice(), &Affine3::identity());  // 2nd borrow mut
        }
    }

    pub fn calculate_transforms(&mut self)
    {
        // isolate roots
        let roots : Vec<_> = self.scene_objects.values().filter(|obj| obj.borrow().parent_id == None).map(|obj| obj.borrow().id).collect();
        self.calculate_transforms_rec(&roots, &Affine3::identity());
    }


    /// Commit the changes made to the scene graph since the last call
    /// They are processed in the order in which they are submitted
    /// TODO do this efficiently (there is a lot of hash lookups)?
    pub fn commit_changes(&mut self) {
        let mut changes = self.changes.borrow_mut();
        for change in changes.drain(..) {
            match change {
                SceneGraphChange::Parent(parent_id,child_id) => {
                    // add child to parent
                    self.scene_objects.get(&parent_id).unwrap().borrow_mut().children.push(child_id);
                    // set parent of child
                    self.scene_objects.get(&child_id).unwrap().borrow_mut().parent_id = Some(parent_id);
                },
                SceneGraphChange::Orphan(parent_id,child_id) => {
                    // remove child from parent
                    self.scene_objects.get(&parent_id).unwrap().borrow_mut().children.retain(|&id| id != child_id);
                    // unset parent from child
                    self.scene_objects.get(&child_id).unwrap().borrow_mut().parent_id = None;
                },
                SceneGraphChange::Insert(scene_object) => {
                    // add child to parent, if the node has a parent
                    if let Some(parent_id) = scene_object.parent_id {
                        self.scene_objects.get(&parent_id).unwrap().borrow_mut().children.push(scene_object.id);
                    }
                    // insert scene object
                    if let Some(_) = self.scene_objects.insert(scene_object.id, RefCell::new(scene_object)) {
                        panic!("Key already present");
                    }
                },
                SceneGraphChange::Remove(ref id) => {
                    self.scene_objects.remove(id).unwrap();
                },
            }
        }
    }
}
