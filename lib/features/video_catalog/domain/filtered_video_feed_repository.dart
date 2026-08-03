import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
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

  Future<Set<ProfileId>> _followedFor(FeedKind kind) {
    return kind == FeedKind.following
        ? _social.loadFollowedProfiles()
        : Future.value(const <ProfileId>{});
  }
}
