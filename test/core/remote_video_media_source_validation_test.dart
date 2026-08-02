import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('rejects non-HTTP primary and fallback video locations', () {
    expect(
      () => VideoMediaSource.remote('file:///tmp/video.mp4'),
      throwsFormatException,
    );
    expect(
      () => VideoMediaSource.remote(
        'https://media.test/video.mp4',
        fallbackUrls: const ['not-a-url'],
      ),
      throwsFormatException,
    );
  });
}
