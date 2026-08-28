import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_interaction_reconciler.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_pagination.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_repost_reconciler.dart';
import 'package:ghostr/features/video_catalog/domain/video_media_identity.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

typedef _FeedResyncLists = ({
  List<VideoPost> refreshed,
  List<VideoPost> eligible,
});

final class FeedSessionResync {
  FeedSessionResync._(
    _FeedResyncLists lists,
    this.retainWatched,
    Iterable<VideoInteractionTarget> retainedHeldTargets,
  ) : refreshed = List.unmodifiable(lists.refreshed),
      eligible = List.unmodifiable(lists.eligible),
      retainedHeldTargets = Set.unmodifiable(retainedHeldTargets);

  final List<VideoPost> refreshed;
  final List<VideoPost> eligible;
  final bool retainWatched;
  final Set<VideoInteractionTarget> retainedHeldTargets;
}

/// The posts one viewing session holds, with the viewer's own interactions
/// layered on top of them.
///
/// Relays report likes and comments late, so every arriving revision is
/// reconciled against what the viewer already did: a stale count never
/// un-likes a video they just liked. The session remembers posts that have
/// scrolled off screen, so they come back reconciled too.
final class FeedSession {
  final _interactions = FeedInteractionReconciler();
  final _reposts = FeedRepostReconciler();
  final _visited = <VideoInteractionTarget>{};
  var _held = const <VideoPost>[];

  /// Every post the session holds, on screen or not.
  List<VideoPost> get held => _held;

  /// A fresh feed: the viewer starts at the top of what arrived, with
  /// later same-video repeats dropped.
  FeedRoster loaded(List<VideoPost> fresh) {
    _visited.clear();
    return _holding(FeedRoster(_reconciled(distinctVideoPosts(fresh), _held)));
  }

  FeedSessionResync captureResync(
    List<VideoPost> refreshed, {
    required List<VideoPost> eligible,
    required bool retainWatched,
  }) {
    return FeedSessionResync._(
      (refreshed: refreshed, eligible: eligible),
      retainWatched,
      _visited,
    );
  }

  FeedRoster previewResynced(FeedRoster visible, FeedSessionResync refresh) {
    return visible.resynced(
      refresh.refreshed,
      eligible: refresh.eligible,
      retainWatched: refresh.retainWatched,
      retainedHeldTargets: refresh.retainedHeldTargets,
    );
  }

  /// A refresh keeps the captured traversal branch and the viewer's place.
  FeedRoster resynced(FeedRoster visible, FeedSessionResync refresh) {
    final current = visible.posts;
    return _holding(
      visible.resynced(
        _reconciled(refresh.refreshed, current),
        eligible: _reconciled(refresh.eligible, current),
        retainWatched: refresh.retainWatched,
        retainedHeldTargets: refresh.retainedHeldTargets,
      ),
    );
  }

  /// Records a video that this surface actually presented to the viewer.
  FeedRoster presentedAt(
    FeedRoster visible,
    int index, {
    required FeedNavigationHistory history,
  }) {
    _rememberTraversal(visible, index, history);
    return _holding(visible.movedTo(index, history: history));
  }

  FeedRoster positionedAt(
    FeedRoster visible,
    int index, {
    required FeedNavigationHistory history,
  }) {
    return _holding(visible.movedTo(index, history: history));
  }

  /// An older page: the posts it adds beyond what the viewer already has,
  /// or null when the page brought nothing new.
  List<VideoPost>? appended(FeedRoster visible, List<VideoPost> page) {
    final posts = FeedPagination.appendNew(
      visible.posts,
      _reconciled(page, const <VideoPost>[]),
    );
    if (posts.length == visible.posts.length) return null;
    _held = posts;
    return posts;
  }

  /// A like the viewer's own tap produced, projected onto the feed.
  List<VideoPost> liked(List<VideoPost> visible, VideoPost accepted) {
    _held = _interactions.acceptLike(accepted, visible);
    return _held;
  }

  List<VideoPost> acceptedRepost(List<VideoPost> visible, VideoPost accepted) {
    _held = _reposts.accept(accepted, visible);
    return _held;
  }

  List<VideoPost> projectedRepost(
    List<VideoPost> visible,
    VideoPost projection,
  ) {
    _held = _reposts.project(projection, visible);
    return _held;
  }

  List<VideoPost> settledReposts(
    List<VideoPost> visible,
    List<VideoPost> settled,
  ) {
    _held = _reposts.settled(settled, visible);
    return _held;
  }

  /// Comments the viewer just published, which raise the count for good.
  List<VideoPost> commented(
    List<VideoPost> visible,
    VideoPost post,
    int count,
  ) {
    _held = _interactions.acceptComments(post, count, visible);
    return _held;
  }

  /// Forgets every post published by a blocked creator.
  void dropCreator(ProfileId creator) {
    _holding(FeedRoster(_held).withoutCreator(creator));
  }

  List<VideoPost> _reconciled(
    List<VideoPost> arriving,
    List<VideoPost> against,
  ) {
    final interactions = _interactions.reconcile(
      refreshed: arriving,
      current: against,
    );
    return _reposts.reconcile(interactions, against);
  }

  FeedRoster _holding(FeedRoster roster) {
    _held = roster.posts;
    final retained = roster.posts.map(VideoInteractionTarget.fromPost).toSet();
    _visited.retainWhere(retained.contains);
    return roster;
  }

  void _rememberTraversal(
    FeedRoster visible,
    int index,
    FeedNavigationHistory history,
  ) {
    final maximum = history.maximumPrevious;
    if (maximum == null) return;
    RangeError.checkValidIndex(index, visible.posts, 'index');
    _visited.add(VideoInteractionTarget.fromPost(visible.active));
    _visited.add(VideoInteractionTarget.fromPost(visible.posts[index]));
    while (_visited.length > maximum + 1) {
      _visited.remove(_visited.first);
    }
  }
}
