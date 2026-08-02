import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

void main() {
  test('rejects blank selected-media strings', () {
    final mime = VideoMimeType.fromFileName('video.mp4');

    expect(
      () => SelectedMedia(
        path: ' ',
        source: MediaPickSource.gallery,
        label: 'video.mp4',
        mimeType: mime,
      ),
      throwsFormatException,
    );
    expect(
      () => SelectedMedia(
        path: '/tmp/video.mp4',
        source: MediaPickSource.gallery,
        label: ' ',
        mimeType: mime,
      ),
      throwsFormatException,
    );
  });
}
