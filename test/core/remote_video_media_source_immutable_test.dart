import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('does not expose mutable remote video locations', () {
    final source = VideoMediaSource.remote(
      'https://media.example/video.mp4',
      fallbackUrls: ['https://backup.example/video.mp4'],
    );

    expect(
      () => source.fallbackUrls[0] = 'https://changed.example/video.mp4',
      throwsUnsupportedError,
    );
    expect(
      () => source.remoteUrls.add('https://changed.example/video.mp4'),
      throwsUnsupportedError,
    );
  });
}
