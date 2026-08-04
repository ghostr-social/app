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
  const FeedViewer({this.focus, this.watchTracker});

  final FeedFocusPort? focus;
  final WatchHistoryTracker? watchTracker;

  /// The viewer landed on `posts[index]` — a load, a swipe, a jump.
  void landedOn(List<VideoPost> posts, int index) {
    _watched(posts[index]);
    // Watch time stays zero: Dart has no playback watch timer yet, so the
    // engine cannot receive real per-post watch milliseconds.
    focus?.focusChanged(
      FeedFocus.around(posts: posts, activeIndex: index),
    );
  }

  /// The viewer stayed on [post] while the list around it changed. The
  /// delivery window is already pointed at it.
  void stayedOn(VideoPost post) => _watched(post);

  void _watched(VideoPost post) {
    final tracker = watchTracker;
    if (tracker != null) unawaited(tracker.videoWatched(post));
  }
}
