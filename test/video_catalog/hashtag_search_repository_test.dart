import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/filtered_video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_reader.dart';

void main() {
  test('routes hashtag queries to the tag filter and plain ones to search',
      () async {
    final reader = _RecordingReader();
    final repository = FilteredVideoSearchRepository(reader, _NoSocialGraph());

    await repository.search('#Dance');
    await repository.search('dance');

    expect(reader.searchQueries, [null, 'dance']);
    expect(reader.hashtagRequests, [
      {'dance'},
      null,
    ]);
  });
}

class _RecordingReader implements VideoPostReader {
  final searchQueries = <String?>[];
  final hashtagRequests = <Set<String>?>[];

  @override
  Future<List<VideoPost>> load({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) async {
    searchQueries.add(searchQuery);
    hashtagRequests.add(hashtags);
    return const <VideoPost>[];
  }
}

class _NoSocialGraph implements SocialGraphRepository {
  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async => const <ProfileId>{};

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async => const <ProfileId>{};

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => false;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => false;
}
