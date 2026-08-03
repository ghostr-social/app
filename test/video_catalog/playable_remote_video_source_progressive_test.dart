import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/video_catalog/data/playable_remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';

void main() {
  test('keeps a progressive post playable', () async {
    final progressive = samplePost();
    final source = PlayableRemoteVideoSource(
      source: FakeRemoteVideoSource([progressive]),
      capabilities: VideoPlaybackCapabilities.progressiveOnly,
    );

    final posts = await source.loadRemoteFeed();

    expect(posts, [progressive]);
  });
}
