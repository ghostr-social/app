import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
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

  const FeedLoaded._(
    super.kind,
    this.posts,
    this.activeIndex,
    this.notice,
  );

  final List<VideoPost> posts;
  final int activeIndex;
  final String? notice;

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
