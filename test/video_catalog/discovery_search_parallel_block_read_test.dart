import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/discovery_video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/discovery_search_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('video search overlaps mute lookup and still excludes blocks', () async {
    final videos = _DeferredVideos();
    final social = _DeferredSocial();
    final harness = DiscoverySearchHarness();
    final repository = DiscoveryVideoSearchRepository(
      videos: videos,
      creators: harness.creators,
      social: social,
      failureReporter: harness.reporter,
    );

    final loading = repository.searchVideos('bitcoin');
    var completed = false;
    loading.whenComplete(() => completed = true);
    await Future<void>.delayed(Duration.zero);

    expect(videos.started, isTrue);
    expect(social.started, isTrue);

    final post = samplePost();
    videos.result.complete([post]);
    await Future<void>.delayed(Duration.zero);
    expect(completed, isFalse);

    social.result.complete({post.creator.id});
    expect((await loading).posts, isEmpty);
  });
}

final class _DeferredVideos extends RecordingRemoteVideoSource {
  _DeferredVideos() : super(const []);

  final result = Completer<List<VideoPost>>();
  bool started = false;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) {
    started = true;
    return result.future;
  }
}

final class _DeferredSocial extends FakeSocialGraph {
  final result = Completer<Set<ProfileId>>();
  bool started = false;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() {
    started = true;
    return result.future;
  }
}
