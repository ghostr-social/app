import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';

/// A feed bound to one creator's published videos, in the order their
/// profile shows them.
///
/// Feed kind and watch exclusion do not apply: the viewer chose this
/// creator's shelf, so every post plays — watched or not. The profile hands
/// over its complete shelf in one load, so the older-page probe reports the
/// past exhausted.
class ProfileVideoFeedRepository implements VideoFeedRepository {
  const ProfileVideoFeedRepository({
    required VideoProfileRepository profile,
    required ProfileSummary viewer,
    required ProfileId creatorId,
  })  : _profile = profile,
        _viewer = viewer,
        _creatorId = creatorId;

  final VideoProfileRepository _profile;
  final ProfileSummary _viewer;
  final ProfileId _creatorId;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    final details = await _profile.loadProfile(_viewer, _creatorId);
    return details.posts;
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    return VideoFeedPage(posts: const <VideoPost>[]);
  }
}
