import 'dart:math' as math;

import 'package:ghostr/features/video_catalog/domain/video_post.dart';

/// The viewer's position in a feed as the delivery engine sees it: an
/// ordered window of posts around — and including — the current item.
final class FeedFocus {
  FeedFocus._(this.window, this.currentIndex, this.watched);

  /// Cuts the engine's focus window out of [posts]: the active post,
  /// up to [aheadCount] posts after it and [behindCount] before it,
  /// in feed order. The sizes mirror the Rust engine's startability
  /// window (next 6) plus its scroll-back neighbours (2).
  factory FeedFocus.around({
    required List<VideoPost> posts,
    required int activeIndex,
    Duration watched = Duration.zero,
  }) {
    RangeError.checkValidIndex(activeIndex, posts, 'activeIndex');
    final start = math.max(0, activeIndex - behindCount);
    final end = math.min(posts.length, activeIndex + aheadCount + 1);
    return FeedFocus._(
      List<VideoPost>.unmodifiable(posts.getRange(start, end)),
      activeIndex - start,
      watched,
    );
  }

  static const int aheadCount = 6;
  static const int behindCount = 2;

  /// Feed-ordered slice around the viewer; always contains [current].
  final List<VideoPost> window;

  /// Index of the current post within [window].
  final int currentIndex;

  /// Time spent watching the current post. Dart has no playback watch
  /// timer yet, so callers pass zero until playback stats exist.
  final Duration watched;

  VideoPost get current => window[currentIndex];
}

/// Domain port announcing where the viewer is, so the media delivery
/// engine can reprioritize downloads around the focus window.
abstract interface class FeedFocusPort {
  /// Replaces the engine's focus window. Implementations must never
  /// throw into the caller — delivery trouble is theirs to absorb.
  void focusChanged(FeedFocus focus);
}
