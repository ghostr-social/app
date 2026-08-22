use crate::api::delivery::focus_mapping::delivery_focus;
use crate::api::delivery_types::FfiMediaDelivery;
use crate::api::focus_control::FfiFocusTransition;
use crate::api::tests::support::ffi_item;
use crate::engine::PreviewDescriptor;

const BLURHASH: &str = "LEHV6nWB2yk8pyo0adR*.7kCMdnj";

#[test]
fn focus_accepts_only_a_valid_inline_blurhash_as_a_preview() {
    let mut valid = ffi_item("valid", FfiMediaDelivery::Progressive);
    valid.blurhash = Some(BLURHASH.to_owned());
    assert_eq!(preview(valid), PreviewDescriptor::inline_blurhash(BLURHASH));

    let mut invalid = ffi_item("invalid", FfiMediaDelivery::Progressive);
    invalid.blurhash = Some("not!blurhash".to_owned());
    assert_eq!(preview(invalid), None);
    assert_eq!(
        preview(ffi_item("absent", FfiMediaDelivery::Progressive)),
        None
    );
}

fn preview(item: crate::api::delivery_types::FfiFocusItem) -> Option<PreviewDescriptor> {
    delivery_focus(&[item], 0, 0, 1, FfiFocusTransition::UserNavigation, None)
        .unwrap()
        .previews
        .first()
        .map(|preview| preview.descriptor)
}
