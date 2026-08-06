import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';

void main() {
  test('normalizes and compares local video file paths', () {
    final first = ShareableVideoFile.parse(' /tmp/video.mp4 ');
    final second = ShareableVideoFile.parse('/tmp/video.mp4');

    expect(first.path, '/tmp/video.mp4');
    expect(first, second);
    expect(first.hashCode, second.hashCode);
  });
}
