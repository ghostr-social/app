import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('validates and normalizes expected remote video SHA-256 digests', () {
    final verified = VideoMediaSource.withExpectedSha256(
      VideoMediaSource.remote('https://media.test/video.mp4'),
      ' AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA ',
    );

    expect(
      verified.expectedSha256?.value,
      'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    );
    expect(verified.debugLabel, 'https://media.test/video.mp4');
    expect(
      () => VideoMediaSource.withExpectedSha256(verified, 'invalid'),
      throwsFormatException,
    );
    expect(
      () => VideoMediaSource.withExpectedSha256(
        VideoMediaSource.local('/video.mp4'),
        'a'.padRight(64, 'a'),
      ),
      throwsFormatException,
    );
  });
}
