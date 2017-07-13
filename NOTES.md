TODO
====

#### NPR rendering primitives
* Gradation offset (means: shader)
* Gradation offset maps
* HQ Temporal AA (no need for reprojection right now)
* Some work on highlights (shape? anisotropy?)
* Normal simplification
* Local modulation
* Silhouette extension (through)
* Deep G-buffers (optional render output)
* Inflated G-buffers (optional render passes)
* Contour maps

#### Interface
* Editor for gradation
* Paint to texture
* Paint vertex colors?
* Assimp importer
* Frame graph
* Shader bits (live editable)
* Save/load 
	* Cameras
	* Objects
	* Shadings
* Undo/Redo + Interface
* Status bar

#### Renderer
* 'Blackboard' with optionally evaluated passes
	* geometry passes
* Main G-buffer pass
* TAA node


#### Shading file format
* JSON/protobuf?
* Layers
* Light dep
* Associated maps: gradations, local maps

#### Idea


#### Scene
HashMap<EntityID, Component>
Operations:
* Collect: remove all expired ids
* Iterate
* Lookup

Limitations:
* Cannot borrow a reference to a component that outlives the frame (collect() may rehash)
* Cannot add new components in place: must do a functional update / staging area for new components (deferred creation)
* Cannot delete components: that's OK (deferred cleanup)
* References must be IDs
