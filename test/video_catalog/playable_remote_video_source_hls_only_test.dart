import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/video_catalog/data/playable_remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';

void main() {
  test('returns an empty feed when every remote post requires HLS', () async {
    final hls = samplePost().withMedia(VideoMediaSource.remote(
      'https://media.example/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    ));
    final source = PlayableRemoteVideoSource(
      source: FakeRemoteVideoSource([hls]),
      capabilities: VideoPlaybackCapabilities.progressiveOnly,
    );

    final posts = await source.loadRemoteFeed();

    expect(posts, isEmpty);
  });
}
