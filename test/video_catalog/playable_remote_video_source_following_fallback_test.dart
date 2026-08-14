import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/video_catalog/data/playable_remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/following_feed_scope_fixture.dart';
import '../support/sample_data.dart';

void main() {
  test('Following operations fall back to generic creator filters', () async {
    final playable = samplePost(id: 'playable');
    final hls = samplePost(id: 'hls').withMedia(
      VideoMediaSource.remote(
        'https://media.example/live.m3u8',
        delivery: VideoMediaDelivery.hls,
      ),
    );
    final remote = FakeRemoteVideoSource([playable, hls])
      ..olderPosts = [playable, hls];
    final source = PlayableRemoteVideoSource(
      source: remote,
      capabilities: VideoPlaybackCapabilities.progressiveOnly,
    );
    final scope = testFollowingFeedScope();
    final cursor = DateTime.utc(2026, 1, 2);

    final first = await source.loadFollowingRemoteFeed(scope);
    final older = await source.loadFollowingRemoteFeed(
      scope,
      olderThan: cursor,
    );
    final snapshot = await source.watchFollowingRemoteFeed(scope).first;

    expect(first, [playable]);
    expect(older, [playable]);
    expect(snapshot.posts, [playable]);
    expect(remote.requestedCreatorIds, scope.creators);
    expect(remote.requestedOlderThan, [cursor]);
    expect(remote.requestedWatchCreatorIds, scope.creators);
  });
}
