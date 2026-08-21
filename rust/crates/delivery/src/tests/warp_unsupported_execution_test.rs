use crate::manager::reconcile_warp::{directive_for, WarpDirective};
use ghostr_engine::adaptive::{PlannerCommand, TransformKind};
use ghostr_engine::{ActionId, PostId};

#[test]
fn unavailable_transform_and_live_promotion_fail_closed() {
    let post = PostId::new("video");
    let transform = PlannerCommand::Transform {
        post: post.clone(),
        kind: TransformKind::Remux,
    };
    let promote = PlannerCommand::Promote {
        post,
        action: ActionId::new(9),
    };

    assert_eq!(
        directive_for(Some(&transform), &[]),
        WarpDirective::Unsupported {
            class: "warp_transform_backend_unavailable",
        }
    );
    assert_eq!(
        directive_for(Some(&promote), &[]),
        WarpDirective::Unsupported {
            class: "warp_live_promotion_backend_unavailable",
        }
    );
}
