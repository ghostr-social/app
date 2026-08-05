import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/video_catalog/data/playable_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';

void main() {
  test('preserves loading phase when playability removes every row', () async {
    final hls = samplePost().withMedia(VideoMediaSource.remote(
      'https://media.example/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    ));
    final remote = FakeRemoteVideoSource([hls])
      ..snapshotPhase = RemoteVideoPhase.loading;
    final source = PlayableRemoteVideoSource(
      source: remote,
      capabilities: VideoPlaybackCapabilities.progressiveOnly,
    );

    final snapshot = await source.watchRemoteFeed().first;

    expect(snapshot.phase, RemoteVideoPhase.loading);
    expect(snapshot.posts, isEmpty);
  });
}
