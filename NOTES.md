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

#### Frame graph: custom derive?

inputs:
    - any kind of resource
        - Image
        - Buffer: CpuAccessibleBuffer (UPLOAD), DeviceLocalBuffer (DEFAULT), ImmutableBuffer (DEFAULT+READONLY)
    - how it should be used in the pass (bound where / resource state)
        - SampledImage, RWImage, Render target attachement, etc.
        - Views:
            BufferView (automatically created?)
            ImageView
        - Clear value
            -
outputs:
    - same as inputs

creates:
    - metadata (width, height, etc.)

execute:
    - access to resources
    - the function should be able to access views of the declared resources in the correct state

pipeline:
    - how should it be managed? can it be dynamically selected?
    - use a lazy_static in the execute() function
    - subpasses automatically created
    - issue: vulkan pipelines must be created with a reference to a subpass
    - layout? push values?

**must be able to recreate the framegraph on each frame**
    - issue: must recreate pipelines
    - the pipelines are valid for one frame only (but likely more than one)

must have an interface to dynamically create passes (variable number of resources, different access flags, etc.)

##### Implementation agenda:
* Explicit API first
    * setup() + execute()
    * No corresponding allocation, just a prototype
*

