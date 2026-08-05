import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/discovery_video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/block_during_fetch_social_graph.dart';
import '../support/discovery_search_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('a block made during search excludes the creator', () async {
    final videos = _PendingVideos();
    final social = BlockDuringFetchSocialGraph();
    final harness = DiscoverySearchHarness();
    final repository = DiscoveryVideoSearchRepository(
      videos: videos,
      creators: harness.creators,
      social: social,
      failureReporter: harness.reporter,
    );
    final post = samplePost();

    final loading = repository.searchVideos('bitcoin');
    await Future<void>.delayed(Duration.zero);
    expect(social.blockedReads, 1);

    await social.toggleBlock(post.creator.id);
    videos.result.complete([post]);

    expect((await loading).posts, isEmpty);
    expect(social.blockedReads, 2);
  });
}

final class _PendingVideos extends RecordingRemoteVideoSource {
  _PendingVideos() : super(const []);

  final result = Completer<List<VideoPost>>();

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) => result.future;
}
