use crate::manager::reconcile_warp::axiom_test_support::directive_for;
use crate::manager::reconcile_warp::WarpDirective;
use ghostr_engine::adaptive::{PlannerCommand, PromotionGrant, TransformKind};
use ghostr_engine::{ActionId, PostId};

#[test]
fn transform_and_promotion_map_their_exact_targets() {
    let post = PostId::new("video");
    let transform = PlannerCommand::Transform {
        post: post.clone(),
        kind: TransformKind::Remux,
    };
    let promote = PlannerCommand::Promote {
        post: post.clone(),
        action: ActionId::new(9),
        source: "https://origin.test/video".into(),
        grant: PromotionGrant {
            maximum_bytes: 16,
            valid_until_ms: 99,
        },
    };

    assert_eq!(
        directive_for(Some(&transform), &[]),
        WarpDirective::Transform {
            post: post.clone(),
            kind: TransformKind::Remux,
        }
    );
    assert_eq!(
        directive_for(Some(&promote), &[]),
        WarpDirective::Promote {
            post,
            action: ActionId::new(9),
            source: "https://origin.test/video".into(),
            grant: PromotionGrant {
                maximum_bytes: 16,
                valid_until_ms: 99,
            },
        }
    );
}
