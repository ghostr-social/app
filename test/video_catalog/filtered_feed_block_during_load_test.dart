import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/filtered_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_reader.dart';

import '../support/block_during_fetch_social_graph.dart';
import '../support/sample_data.dart';

void main() {
  test('a block made during feed retrieval excludes the creator', () async {
    final reader = _PendingReader();
    final social = BlockDuringFetchSocialGraph();
    final repository = FilteredVideoFeedRepository(reader, social);
    final post = samplePost();

    final loading = repository.loadFeed(FeedKind.forYou);
    await Future<void>.delayed(Duration.zero);
    expect(social.blockedReads, 1);

    await social.toggleBlock(post.creator.id);
    reader.result.complete([post]);

    expect(await loading, isEmpty);
    expect(social.blockedReads, 2);
  });
}

final class _PendingReader implements VideoPostReader {
  final result = Completer<List<VideoPost>>();

  @override
  Future<List<VideoPost>> load({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) => result.future;

  @override
  Future<List<VideoPost>> loadOlder({
    required DateTime olderThan,
    Set<ProfileId>? creatorIds,
  }) async => const <VideoPost>[];
}
