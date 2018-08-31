use super::*;

fn hit_test_node(nodes: &Arena<RetainedNode>, id: NodeId, pos: (f32, f32), hits: &mut Vec<NodeId>)
{
    let node = &nodes[id];
    let data = node.data();
    let hit = data.layout.is_point_inside(pos);
    if hit || !data.clip_info.no_hit {
        hits.push(id);
    }
    if hit || !data.clip_info.clip {
        // collect nodes
        let mut children = Vec::new();
        let mut next = node.first_child();

        while let Some(id) = next {
            let node = nodes[id];
            children.push((node.data().clip_info.z, id));
            next = node.next_sibling();
        }

        // hit test children in local z-order
        children.sort_by(|a, b| { a.0.cmp(b.0) });

        for c in children {
            hit_test_node(nodes, c.1, pos, hits);
        }
    }
}

pub fn hit_test_dom(nodes: &Arena<RetainedNode>, root: NodeId, pos: (f32, f32)) -> Vec < NodeId >
{
    let mut hits = Vec::new();
    //let root = &nodes[root];
    hit_test_node(nodes, root, pos, &mut hits);
    hits
}

