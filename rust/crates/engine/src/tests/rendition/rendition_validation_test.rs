use crate::rendition::{Rendition, RenditionError, RenditionId, RenditionSet, RenditionSetError};
use crate::tests::rendition_support::rendition;

#[test]
fn invalid_or_ambiguous_rendition_ladders_are_rejected_at_construction() {
    assert_eq!(RenditionId::try_new(""), Err(RenditionError::EmptyId));
    assert_eq!(RenditionId::try_new("  "), Err(RenditionError::EmptyId));
    assert_eq!(
        Rendition::try_new("zero", 0),
        Err(RenditionError::ZeroBitrate),
    );
    assert_eq!(RenditionSet::try_new(vec![]), Err(RenditionSetError::Empty));
    assert_eq!(
        RenditionSet::try_new(vec![rendition("same", 1), rendition("same", 2)]),
        Err(RenditionSetError::DuplicateId),
    );
    assert_eq!(
        RenditionSet::try_new(vec![rendition("first", 1), rendition("second", 1)]),
        Err(RenditionSetError::DuplicateBitrate),
    );
}
