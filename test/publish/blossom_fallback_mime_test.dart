import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/blossom_upload_result_mapper.dart';
import 'package:ndk/ndk.dart';

void main() {
  test('uses the selected video MIME type when Blossom omits it', () {
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
            sha256:
                'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
            size: 10,
            uploaded: DateTime(2026, 8, 2),
          ),
        ),
      ],
    );

    final media = const BlossomUploadResultMapper().map(
      progress,
      fallbackMimeType: 'video/mp4',
    );

    expect(media.mimeType, 'video/mp4');
  });
}
