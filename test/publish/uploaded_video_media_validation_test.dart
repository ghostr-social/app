import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/publish/domain/uploaded_video_media.dart';

void main() {
  const digest =
      'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';

  test('rejects invalid uploaded-video metadata at construction', () {
    expect(
      () => VideoUploadMetadata(
        sha256: 'not-a-digest',
        mimeType: 'video/mp4',
        sizeBytes: 1,
      ),
      throwsFormatException,
    );
    expect(
      () => VideoUploadMetadata(
        sha256: digest,
        mimeType: 'text/plain',
        sizeBytes: 1,
      ),
      throwsFormatException,
    );
    expect(
      () => VideoUploadMetadata(
        sha256: digest,
        mimeType: 'video/mp4',
        sizeBytes: 0,
      ),
      throwsFormatException,
    );
  });
}
