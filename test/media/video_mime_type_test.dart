import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

void main() {
  test('derives a supported video MIME type from a file name', () {
    expect(VideoMimeType.fromFileName('clip.MP4').value, 'video/mp4');
    expect(VideoMimeType.fromFileName('clip.webm').value, 'video/webm');
    expect(VideoMimeType.tryParse('image/png'), isNull);
  });
}
