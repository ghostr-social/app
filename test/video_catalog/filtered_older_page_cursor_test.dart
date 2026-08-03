import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/filtered_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_reader.dart';

import '../support/sample_data.dart';

void main() {
  test('a page of blocked creators still advances the cursor', () async {
    final blocked = sampleCreator(id: 'creator-blocked');
    final oldest = DateTime.utc(2026, 7, 1);
    final reader = _PageReader([
      samplePost(id: 'post-1', creator: blocked, publishedAt: oldest),
      samplePost(
        id: 'post-2',
        creator: blocked,
        publishedAt: DateTime.utc(2026, 7, 2),
      ),
    ]);
    final feed = FilteredVideoFeedRepository(reader, _Social({blocked.id}));

    final page = await feed.loadOlderFeed(
      FeedKind.forYou,
      olderThan: DateTime.utc(2026, 7, 3),
    );

    expect(page.posts, isEmpty);
    expect(page.nextOlderThan, oldest.subtract(const Duration(seconds: 1)));
  });
}

class _PageReader implements VideoPostReader {
  _PageReader(this.page);

  final List<VideoPost> page;

  @override
  Future<List<VideoPost>> load({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) async {
    return const <VideoPost>[];
  }

  @override
  Future<List<VideoPost>> loadOlder({
    required DateTime olderThan,
    Set<ProfileId>? creatorIds,
  }) async {
    return page;
  }
}

class _Social implements SocialGraphRepository {
  _Social(this.blocked);

  final Set<ProfileId> blocked;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async => blocked;

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async => const <ProfileId>{};

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => true;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => true;
}
