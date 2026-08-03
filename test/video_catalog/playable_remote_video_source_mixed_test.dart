import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/video_catalog/data/playable_remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';

void main() {
  test('keeps progressive posts in their original order while dropping HLS',
      () async {
    final first = samplePost(id: 'first');
    final hls = samplePost(id: 'hls').withMedia(VideoMediaSource.remote(
      'https://media.example/hls.m3u8',
      delivery: VideoMediaDelivery.hls,
    ));
    final last = samplePost(id: 'last');
    final source = PlayableRemoteVideoSource(
      source: FakeRemoteVideoSource([first, hls, last]),
      capabilities: VideoPlaybackCapabilities.progressiveOnly,
    );

    final posts = await source.loadRemoteFeed();

    expect(posts, [first, last]);
  });
}
