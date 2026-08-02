import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/blossom_upload_result_mapper.dart';
import 'package:ndk/ndk.dart';

void main() {
  test('rejects a Blossom descriptor without a SHA-256 digest', () {
    final progress = BlobUploadProgress(
      currentServer: '',
      sentBytes: 10,
      totalBytes: 10,
      isComplete: true,
      completedUploads: [
        BlobUploadResult(
          serverUrl: 'https://media.example',
          success: true,
          descriptor: BlobDescriptor(
            url: 'https://media.example/video.mp4',
            sha256: 'not-a-sha256',
            size: 10,
            uploaded: DateTime(2026, 8, 2),
            type: 'video/mp4',
          ),
        ),
      ],
    );

    expect(
      () => const BlossomUploadResultMapper().map(
        progress,
        fallbackMimeType: 'video/mp4',
      ),
      throwsA(isA<Exception>()),
    );
  });
}
