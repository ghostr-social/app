use crate::manager::cooldown_timers::CooldownTimers;
use crate::manager::retry::{CooldownId, RetryBook, RetryPolicy};
use crate::manager::transfers::{InternalEvent, MaintenanceEvent};
use core::time::Duration;
use ghostr_engine::PostId;
use std::collections::HashSet;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

#[tokio::test(start_paused = true)]
async fn replacement_aborts_the_old_timer_and_keeps_one_owner() {
    let post = PostId::new("playing");
    let mut retry = RetryBook::new(RetryPolicy::default());
    let (events, mut inbox) = unbounded_channel();
    let mut timers = CooldownTimers::default();
    let first = retry.cool_down(post.clone()).expect("valid test fixture");
    timers.start(post.clone(), first, Duration::from_secs(5), events.clone());
    retry.focus_changed(None, Some(&post));
    let second = retry.cool_down(post.clone()).expect("valid test fixture");

    timers.start(post.clone(), second, Duration::from_secs(10), events);

    assert_eq!(timers.len(), 1);
    tokio::time::advance(Duration::from_secs(5)).await;
    tokio::task::yield_now().await;
    assert!(inbox.try_recv().is_err(), "replaced timer still fired");
    tokio::time::advance(Duration::from_secs(5)).await;
    assert_eq!(recv_cooldown(&mut inbox).await, (post.clone(), second));
    assert!(timers.finish(&post, second));
    assert_eq!(timers.len(), 0);
}

#[tokio::test(start_paused = true)]
async fn stale_expiry_cannot_remove_a_replacement_timer() {
    let post = PostId::new("playing");
    let mut retry = RetryBook::new(RetryPolicy::default());
    let (events, mut inbox) = unbounded_channel();
    let mut timers = CooldownTimers::default();
    let stale = retry.cool_down(post.clone()).expect("valid test fixture");
    timers.start(post.clone(), stale, Duration::ZERO, events.clone());
    tokio::task::yield_now().await;
    assert_eq!(recv_cooldown(&mut inbox).await, (post.clone(), stale));
    retry.focus_changed(None, Some(&post));
    let current = retry.cool_down(post.clone()).expect("valid test fixture");
    timers.start(post.clone(), current, Duration::from_secs(5), events);

    assert!(!timers.finish(&post, stale));
    assert_eq!(timers.len(), 1);
    tokio::time::advance(Duration::from_secs(5)).await;
    assert_eq!(recv_cooldown(&mut inbox).await, (post.clone(), current));
}

#[tokio::test(start_paused = true)]
async fn retention_and_clear_abort_evicted_timers() {
    let old = PostId::new("old");
    let kept = PostId::new("kept");
    let mut retry = RetryBook::new(RetryPolicy::default());
    let (events, mut inbox) = unbounded_channel();
    let mut timers = CooldownTimers::default();
    let old_id = retry.cool_down(old.clone()).expect("valid test fixture");
    let kept_id = retry.cool_down(kept.clone()).expect("valid test fixture");
    timers.start(old, old_id, Duration::from_secs(5), events.clone());
    timers.start(kept.clone(), kept_id, Duration::from_secs(5), events);

    timers.retain(&HashSet::from([kept.clone()]));
    assert_eq!(timers.len(), 1);
    tokio::time::advance(Duration::from_secs(5)).await;
    assert_eq!(recv_cooldown(&mut inbox).await, (kept, kept_id));
    timers.clear();
    assert_eq!(timers.len(), 0);
    assert!(inbox.try_recv().is_err(), "evicted timer still fired");
}

#[tokio::test(start_paused = true)]
async fn cancellation_aborts_the_owned_timer() {
    let post = PostId::new("playing");
    let mut retry = RetryBook::new(RetryPolicy::default());
    let (events, mut inbox) = unbounded_channel();
    let mut timers = CooldownTimers::default();
    let cooldown = retry.cool_down(post.clone()).expect("valid test fixture");
    timers.start(post.clone(), cooldown, Duration::from_secs(5), events);

    timers.cancel(&post);

    assert_eq!(timers.len(), 0);
    tokio::time::advance(Duration::from_secs(5)).await;
    tokio::task::yield_now().await;
    assert!(inbox.try_recv().is_err(), "cancelled timer still fired");
}

async fn recv_cooldown(inbox: &mut UnboundedReceiver<InternalEvent>) -> (PostId, CooldownId) {
    match inbox.recv().await.expect("valid test fixture") {
        InternalEvent::Maintenance(MaintenanceEvent::CooldownOver(post, cooldown)) => {
            (post, cooldown)
        }
        _ => panic!("unexpected timer event"),
    }
}
