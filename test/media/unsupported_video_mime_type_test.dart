import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

void main() {
  test('rejects a file name without a supported video extension', () {
    expect(
      () => VideoMimeType.fromFileName('poster.png'),
      throwsFormatException,
    );
  });
}
