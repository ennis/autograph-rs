use aabb::AABB;
use id_table::ID;
use mesh::{Mesh, Vertex3};
use nalgebra::Affine3;
use std::cell::RefCell;
use std::collections::hash_map;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct SceneMesh {
    pub mesh: Mesh<Vertex3>,
    pub aabb: AABB<f32>,
}

///
///
pub struct SceneObject {
    pub id: ID,
    pub name: String,
    pub parent_id: Option<ID>,
    pub local_transform: Affine3<f32>,
    pub world_transform: Affine3<f32>,
    pub world_bounds: AABB<f32>, // TODO Should be Option<AABB<f32>>, since an object may not have world bounds
    pub children: Vec<ID>,
    pub mesh: Option<Arc<SceneMesh>>, // TODO should this be in its own component map?
}

impl SceneObject {}

enum SceneGraphChange {
    Parent(ID, ID),
    Orphan(ID, ID),
    Insert(SceneObject),
    Remove(ID),
}

pub struct SceneObjects {
    scene_objects: HashMap<ID, RefCell<SceneObject>>,
    changes: RefCell<Vec<SceneGraphChange>>,
}

/// The actual scene graph is changed at the end of the frame
/// This way the pointers to the scene objects stay stable within a frame
impl SceneObjects {
    pub fn new() -> SceneObjects {
        SceneObjects {
            scene_objects: HashMap::new(),
            changes: RefCell::new(Vec::new()),
        }
    }

    pub fn iter(&self) -> hash_map::Iter<ID, RefCell<SceneObject>> {
        self.scene_objects.iter()
    }

    /// Add a parent/child relationship between the two IDs
    pub fn parent(&self, parent: ID, child: ID) {
        self.changes
            .borrow_mut()
            .push(SceneGraphChange::Parent(parent, child));
    }

    pub fn remove(&self, id: ID) {
        self.changes.borrow_mut().push(SceneGraphChange::Remove(id));
    }

    pub fn orphan(&self, parent: ID, child: ID) {
        self.changes
            .borrow_mut()
            .push(SceneGraphChange::Orphan(parent, child));
    }

    pub fn insert(&self, scene_object: SceneObject) {
        self.changes
            .borrow_mut()
            .push(SceneGraphChange::Insert(scene_object));
    }

    fn calculate_transforms_rec(&self, ids: &[ID], parent_transform: &Affine3<f32>) {
        for id in ids.iter() {
            let mut so = self.scene_objects.get(&id).unwrap().borrow_mut(); // borrow mut self
            so.world_transform = parent_transform * so.local_transform;
            self.calculate_transforms_rec(so.children.as_slice(), &Affine3::identity()); // 2nd borrow mut
        }
    }

    fn calculate_bounds_rec(&self, id: ID) -> AABB<f32> {
        // get scene object
        let mut so = self.scene_objects.get(&id).unwrap().borrow_mut();
        // compute local bounds
        let mut bounds = if let Some(ref sm) = so.mesh {
            sm.aabb.transform(&so.world_transform)
        } else {
            AABB::empty()
        };
        // Union with all child elements
        for c in so.children.iter() {
            bounds.union_with(&self.calculate_bounds_rec(*c));
        }
        so.world_bounds = bounds;
        bounds
    }

    pub fn get(&self, id: ID) -> Option<&RefCell<SceneObject>> {
        self.scene_objects.get(&id)
    }

    pub fn calculate_transforms(&mut self) {
        // isolate roots
        let roots: Vec<_> = self.scene_objects
            .values()
            .filter(|obj| obj.borrow().parent_id == None)
            .map(|obj| obj.borrow().id)
            .collect();
        //debug!("calculate_transforms: {} roots", roots.len());
        self.calculate_transforms_rec(&roots, &Affine3::identity());
        // now update bounds
        for r in roots {
            self.calculate_bounds_rec(r);
        }
    }

    /// Commit the changes made to the scene graph since the last call
    /// They are processed in the order in which they are submitted
    /// TODO do this efficiently (there is a lot of hash lookups)?
    pub fn commit_changes(&mut self) {
        let mut changes = self.changes.borrow_mut();
        for change in changes.drain(..) {
            match change {
                SceneGraphChange::Parent(parent_id, child_id) => {
                    debug!("parenting {:?} -> {:?}", parent_id, child_id);
                    // add child to parent
                    self.scene_objects
                        .get(&parent_id)
                        .unwrap()
                        .borrow_mut()
                        .children
                        .push(child_id);
                    // set parent of child
                    self.scene_objects
                        .get(&child_id)
                        .unwrap()
                        .borrow_mut()
                        .parent_id = Some(parent_id);
                }
                SceneGraphChange::Orphan(parent_id, child_id) => {
                    debug!("orphaning {:?} -> {:?}", parent_id, child_id);
                    // remove child from parent
                    self.scene_objects
                        .get(&parent_id)
                        .unwrap()
                        .borrow_mut()
                        .children
                        .retain(|&id| id != child_id);
                    // unset parent from child
                    self.scene_objects
                        .get(&child_id)
                        .unwrap()
                        .borrow_mut()
                        .parent_id = None;
                }
                SceneGraphChange::Insert(scene_object) => {
                    debug!("inserting {:?}", scene_object.id);
                    // add child to parent, if the node has a parent
                    if let Some(parent_id) = scene_object.parent_id {
                        self.scene_objects
                            .get(&parent_id)
                            .unwrap()
                            .borrow_mut()
                            .children
                            .push(scene_object.id);
                    }
                    // insert scene object
                    if let Some(_) = self.scene_objects
                        .insert(scene_object.id, RefCell::new(scene_object))
                    {
                        panic!("Key already present");
                    }
                }
                SceneGraphChange::Remove(id) => {
                    debug!("removing {:?}", id);
                    self.scene_objects.remove(&id).unwrap();
                }
            }
        }
    }
}
