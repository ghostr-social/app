import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/video_catalog/data/playable_remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';

void main() {
  test('load-more forwards its filter and keeps supported media only',
      () async {
    final progressive = samplePost(id: 'progressive');
    final hls = samplePost(id: 'hls').withMedia(VideoMediaSource.remote(
      'https://media.example/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    ));
    final remote = FakeRemoteVideoSource(const [])
      ..olderPosts = [progressive, hls];
    final source = PlayableRemoteVideoSource(
      source: remote,
      capabilities: VideoPlaybackCapabilities.progressiveOnly,
    );

    final posts = await source.loadMoreRemoteFeed(
      searchQuery: 'ghost',
      hashtags: const {'dance'},
    );

    expect(posts.single.id.value, 'progressive');
    expect(remote.requestedLoadMoreSearchQuery, 'ghost');
    expect(remote.requestedLoadMoreHashtags, {'dance'});
  });
}
