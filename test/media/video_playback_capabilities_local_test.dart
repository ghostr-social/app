import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';

void main() {
  test('progressive playback capabilities retain local media', () {
    final local = VideoMediaSource.local('/videos/draft.mp4');

    expect(VideoPlaybackCapabilities.progressiveOnly.supports(local), isTrue);
  });
}
