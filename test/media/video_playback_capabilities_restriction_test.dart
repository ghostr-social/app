import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';

void main() {
  test('removing HLS preserves local and progressive playback', () {
    final capabilities = VideoPlaybackCapabilities.progressiveAndHls.without(
      VideoMediaDelivery.hls,
    );
    final local = VideoMediaSource.local('/videos/draft.mp4');
    final progressive = VideoMediaSource.remote(
      'https://media.example/video.mp4',
    );
    final hls = VideoMediaSource.remote(
      'https://media.example/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    );

    expect(capabilities.supportsAny, isTrue);
    expect(capabilities.supports(local), isTrue);
    expect(capabilities.supports(progressive), isTrue);
    expect(capabilities.supports(hls), isFalse);
  });
}
