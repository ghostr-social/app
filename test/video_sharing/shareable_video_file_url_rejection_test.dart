import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';

void main() {
  test('rejects a remote URL as a shareable file', () {
    expect(
      () => ShareableVideoFile.parse('https://media.test/video.mp4'),
      throwsFormatException,
    );
  });
}
