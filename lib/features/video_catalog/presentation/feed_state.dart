import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

sealed class FeedState {
  const FeedState(this.kind);

  final FeedKind kind;
}

class FeedLoading extends FeedState {
  const FeedLoading(super.kind);
}

class FeedEmpty extends FeedState {
  const FeedEmpty(super.kind);
}

class FeedFailure extends FeedState {
  const FeedFailure(super.kind, this.message);

  final String message;
}

class FeedLoaded extends FeedState {
  factory FeedLoaded(
    FeedKind kind,
    List<VideoPost> posts, {
    int activeIndex = 0,
    String? notice,
  }) {
    if (posts.isEmpty) throw StateError('Loaded feed cannot be empty.');
    RangeError.checkValidIndex(activeIndex, posts, 'activeIndex');
    return FeedLoaded._(
      kind,
      List<VideoPost>.unmodifiable(posts),
      activeIndex,
      notice,
    );
  }

  /// The state showing [roster]: its posts, with the viewer standing where
  /// the roster left them.
  factory FeedLoaded.of(FeedKind kind, FeedRoster roster, {String? notice}) {
    return FeedLoaded(
      kind,
      roster.posts,
      activeIndex: roster.activeIndex,
      notice: notice,
    );
  }

  const FeedLoaded._(
    super.kind,
    this.posts,
    this.activeIndex,
    this.notice,
  );

  final List<VideoPost> posts;
  final int activeIndex;
  final String? notice;

  /// What the viewer is scrolling through and where they are standing.
  FeedRoster get roster => FeedRoster(posts, activeIndex: activeIndex);

  FeedLoaded withPage(int index) {
    return FeedLoaded(kind, posts, activeIndex: index, notice: notice);
  }

  FeedLoaded withPosts(List<VideoPost> updated) {
    return FeedLoaded(kind, updated, activeIndex: activeIndex);
  }

  FeedLoaded withNotice(String message) {
    return FeedLoaded(kind, posts, activeIndex: activeIndex, notice: message);
  }

  FeedLoaded withoutNotice() {
    return FeedLoaded(kind, posts, activeIndex: activeIndex);
  }
}
