import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/publish/domain/uploaded_video_media.dart';

void main() {
  test('requires uploaded video media to use a remote source', () {
    final metadata = VideoUploadMetadata(
      sha256: 'a' * 64,
      mimeType: 'video/mp4',
      sizeBytes: 1,
    );

    expect(
      () => UploadedVideoMedia(
        source: VideoMediaSource.local('/tmp/video.mp4'),
        metadata: metadata,
      ),
      throwsFormatException,
    );
  });
}
