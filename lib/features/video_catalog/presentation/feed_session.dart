import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_interaction_reconciler.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_pagination.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';

/// The posts one viewing session holds, with the viewer's own interactions
/// layered on top of them.
///
/// Relays report likes and comments late, so every arriving revision is
/// reconciled against what the viewer already did: a stale count never
/// un-likes a video they just liked. The session remembers posts that have
/// scrolled off screen, so they come back reconciled too.
final class FeedSession {
  final _interactions = FeedInteractionReconciler();
  var _held = const <VideoPost>[];

  /// Every post the session holds, on screen or not.
  List<VideoPost> get held => _held;

  /// A fresh feed: the viewer starts at the top of what arrived.
  FeedRoster loaded(List<VideoPost> fresh) {
    return _holding(FeedRoster(_reconciled(fresh, _held)));
  }

  /// A refresh: [visible] keeps its order and the viewer keeps their place.
  FeedRoster resynced(FeedRoster visible, List<VideoPost> refreshed) {
    return _holding(visible.resynced(_reconciled(refreshed, visible.posts)));
  }

  /// An older page: the posts it adds beyond what the viewer already has,
  /// or null when the page brought nothing new.
  List<VideoPost>? appended(FeedState feed, List<VideoPost> page) {
    final visible = _visible(feed);
    final posts = FeedPagination.appendNew(
      visible,
      _reconciled(page, const <VideoPost>[]),
    );
    if (posts.length == visible.length) return null;
    _held = posts;
    return posts;
  }

  /// A like the viewer's own tap produced, projected onto the feed.
  List<VideoPost> liked(FeedState feed, VideoPost accepted) {
    _held = _interactions.acceptLike(accepted, _visible(feed));
    return _held;
  }

  /// Comments the viewer just published, which raise the count for good.
  List<VideoPost> commented(FeedState feed, VideoPost post, int count) {
    _held = _interactions.acceptComments(post, count, _visible(feed));
    return _held;
  }

  /// Forgets every post published by a blocked creator.
  void dropCreator(ProfileId creator) {
    _held = FeedRoster(_held).withoutCreator(creator).posts;
  }

  List<VideoPost> _reconciled(
    List<VideoPost> arriving,
    List<VideoPost> against,
  ) {
    return _interactions.reconcile(refreshed: arriving, current: against);
  }

  List<VideoPost> _visible(FeedState feed) {
    return feed is FeedLoaded ? feed.posts : _held;
  }

  FeedRoster _holding(FeedRoster roster) {
    _held = roster.posts;
    return roster;
  }
}
