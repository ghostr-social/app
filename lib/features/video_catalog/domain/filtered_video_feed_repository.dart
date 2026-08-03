import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_policy.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_reader.dart';

class FilteredVideoFeedRepository implements VideoFeedRepository {
  const FilteredVideoFeedRepository(
    this._reader,
    this._social, {
    VideoFeedPolicy policy = const VideoFeedPolicy(),
  }) : _policy = policy;

  final VideoPostReader _reader;
  final SocialGraphRepository _social;
  final VideoFeedPolicy _policy;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    final followed = await _followedFor(kind);
    final posts = await _reader.load(
      creatorIds: kind == FeedKind.following ? followed : null,
    );
    return _policy.select(
      kind: kind,
      posts: posts,
      followed: followed,
      blocked: await _social.loadBlockedProfiles(),
    );
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    final followed = await _followedFor(kind);
    final posts = await _reader.loadOlder(
      olderThan: olderThan,
      creatorIds: kind == FeedKind.following ? followed : null,
    );
    final selected = _policy.select(
      kind: kind,
      posts: posts,
      followed: followed,
      blocked: await _social.loadBlockedProfiles(),
    );
    // The cursor advances by what was fetched, not what survived filtering,
    // so pages full of blocked creators cannot stall pagination.
    return VideoFeedPage(posts: selected, nextOlderThan: _nextCursor(posts));
  }

  DateTime? _nextCursor(List<VideoPost> fetched) {
    if (fetched.isEmpty) return null;
    var oldest = fetched.first.publishedAt;
    for (final post in fetched.skip(1)) {
      if (post.publishedAt.isBefore(oldest)) oldest = post.publishedAt;
    }
    return oldest.subtract(const Duration(seconds: 1));
  }

  Future<Set<ProfileId>> _followedFor(FeedKind kind) {
    return kind == FeedKind.following
        ? _social.loadFollowedProfiles()
        : Future.value(const <ProfileId>{});
  }
}
