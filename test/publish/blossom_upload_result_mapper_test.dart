import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/nostr/blossom_upload_result_mapper.dart';
import 'package:ndk/ndk.dart';

void main() {
  test('maps successful Blossom uploads into primary and fallback media', () {
    final progress = BlobUploadProgress(
      currentServer: '',
      sentBytes: 4096,
      totalBytes: 4096,
      isComplete: true,
      completedUploads: [
        _success('https://one.example/abc.mp4'),
        _success('https://two.example/abc.mp4'),
      ],
    );

    final media = const BlossomUploadResultMapper().map(
      progress,
      fallbackMimeType: 'video/mp4',
    );

    expect(media.primaryUrl, 'https://one.example/abc.mp4');
    expect(media.fallbackUrls, ['https://two.example/abc.mp4']);
    expect(
      media.sha256,
      'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    );
    expect(media.mimeType, 'video/mp4');
    expect(media.sizeBytes, 4096);
  });
}

BlobUploadResult _success(String url) {
  return BlobUploadResult(
    serverUrl: Uri.parse(url).origin,
    success: true,
    descriptor: BlobDescriptor(
      url: url,
      sha256:
          'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
      size: 4096,
      uploaded: DateTime(2026, 8, 2),
      type: 'video/mp4',
    ),
  );
}
