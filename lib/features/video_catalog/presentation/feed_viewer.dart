import 'dart:async';

import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

/// Tells the rest of the app where the viewer is.
///
/// The delivery engine reprioritizes its downloads around the focus window,
/// and watch history remembers the video on screen so it is not served
/// again. Both are optional: a feed can run without either.
final class FeedViewer {
  FeedViewer({this.focus, this.watchTracker});

  final FeedFocusPort? focus;
  final WatchHistoryTracker? watchTracker;

  /// The viewer landed on `posts[index]` — a load, a swipe, a jump.
  void landedOn(List<VideoPost> posts, int index) {
    _watched(posts[index]);
    rosterChanged(posts, index);
  }

  /// The visible roster changed while the viewer remained in place.
  void rosterChanged(List<VideoPost> posts, int index) {
    _publish(posts, index);
  }

  void visibilityChanged(bool isVisible) {
    final lease = focus;
    if (lease is! FeedFocusLease) return;
    if (!isVisible) return lease.deactivate();
    lease.activate();
  }

  void dispose() {
    final lease = focus;
    if (lease is FeedFocusLease) lease.release();
  }

  void _publish(List<VideoPost> posts, int index) {
    // Reactivation carries no invented watch time.
    focus?.focusChanged(FeedFocus.around(posts: posts, activeIndex: index));
  }

  void _watched(VideoPost post) {
    final tracker = watchTracker;
    if (tracker != null) unawaited(tracker.videoWatched(post));
  }
}
