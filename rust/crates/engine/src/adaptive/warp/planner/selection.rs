use crate::adaptive::{ActionNode, GeneratedAction, SearchDecision};

pub(super) fn selected_action(
    generated: &[GeneratedAction],
    search: &SearchDecision,
) -> Option<GeneratedAction> {
    let id = search.action.as_ref()?.id;
    generated.iter().find(|item| item.node.id == id).cloned()
}

pub(super) fn pruned_ids(generated: &[GeneratedAction], retained: &[ActionNode]) -> Vec<u16> {
    generated
        .iter()
        .filter(|item| !retained.iter().any(|node| node.id == item.node.id))
        .map(|item| item.node.id)
        .collect()
}
