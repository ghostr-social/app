import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_follow_state.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_preparation_reducer.dart';

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

final class FeedRosterRevision {
  FeedRosterRevision();
}

class FeedLoaded extends FeedState {
  factory FeedLoaded(
    FeedKind kind,
    List<VideoPost> posts, {
    int activeIndex = 0,
    String? notice,
    FeedFollowState? follows,
  }) {
    if (posts.isEmpty) throw StateError('Loaded feed cannot be empty.');
    RangeError.checkValidIndex(activeIndex, posts, 'activeIndex');
    return FeedLoaded._(
      kind,
      List<VideoPost>.unmodifiable(posts),
      activeIndex,
      FeedRosterRevision(),
      _FeedLoadedPresentation(
        notice,
        follows ?? FeedFollowState.unavailable(),
        FeedPlaybackPreparation.unmanaged(),
      ),
    );
  }

  /// The state showing [roster]: its posts, with the viewer standing where
  /// the roster left them.
  factory FeedLoaded.of(
    FeedKind kind,
    FeedRoster roster, {
    String? notice,
    FeedFollowState? follows,
  }) {
    return FeedLoaded(
      kind,
      roster.posts,
      activeIndex: roster.activeIndex,
      notice: notice,
      follows: follows,
    );
  }

  const FeedLoaded._(
    super.kind,
    this.posts,
    this.activeIndex,
    this.rosterRevision,
    this._presentation,
  );

  final List<VideoPost> posts;
  final int activeIndex;
  final FeedRosterRevision rosterRevision;
  final _FeedLoadedPresentation _presentation;

  String? get notice => _presentation.notice;
  FeedFollowState get follows => _presentation.follows;
  FeedPlaybackPreparation get preparation => _presentation.preparation;

  bool canFollow(ProfileId profileId) => follows.canFollow(profileId);

  /// What the viewer is scrolling through and where they are standing.
  FeedRoster get roster => FeedRoster(posts, activeIndex: activeIndex);

  FeedLoaded withPage(int index) {
    return FeedLoaded._(kind, posts, index, rosterRevision, _presentation);
  }

  FeedLoaded withPosts(List<VideoPost> updated) {
    return FeedLoaded._(
      kind,
      updated,
      activeIndex,
      rosterRevision,
      _presentation,
    );
  }

  FeedLoaded withNotice(String message) {
    return FeedLoaded._(
      kind,
      posts,
      activeIndex,
      rosterRevision,
      _presentation.withNotice(message),
    );
  }

  FeedLoaded withoutNotice() {
    return FeedLoaded._(
      kind,
      posts,
      activeIndex,
      rosterRevision,
      _presentation.withNotice(null),
    );
  }

  FeedLoaded withFollows(FeedFollowState updated) {
    return FeedLoaded._(
      kind,
      posts,
      activeIndex,
      rosterRevision,
      _presentation.withFollows(updated),
    );
  }

  FeedLoaded withPreparation(FeedPlaybackPreparation updated) {
    return FeedLoaded._(
      kind,
      posts,
      activeIndex,
      rosterRevision,
      _presentation.withPreparation(updated),
    );
  }
}

final class _FeedLoadedPresentation {
  const _FeedLoadedPresentation(this.notice, this.follows, this.preparation);

  final String? notice;
  final FeedFollowState follows;
  final FeedPlaybackPreparation preparation;

  _FeedLoadedPresentation withNotice(String? updated) {
    return _FeedLoadedPresentation(updated, follows, preparation);
  }

  _FeedLoadedPresentation withFollows(FeedFollowState updated) {
    return _FeedLoadedPresentation(notice, updated, preparation);
  }

  _FeedLoadedPresentation withPreparation(FeedPlaybackPreparation updated) {
    return _FeedLoadedPresentation(notice, follows, updated);
  }
}
