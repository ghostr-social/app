use crate::adaptive::ActionNode;
use std::collections::BTreeSet;

pub(super) fn satisfied(mut nodes: Vec<ActionNode>) -> Vec<ActionNode> {
    loop {
        let ids: BTreeSet<_> = nodes.iter().map(|node| node.id).collect();
        let before = nodes.len();
        nodes.retain(|node| node.requires.iter().all(|id| ids.contains(id)));
        if nodes.len() == before {
            return nodes;
        }
    }
}
