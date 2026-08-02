import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ghostr/platform/nostr/ndk_blossom_video_uploader.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/sample_data.dart';

void main() {
  test('uploads selected video bytes to every configured Blossom server',
      () async {
    final ndk = MockNdk();
    final files = MockFiles();
    when(() => ndk.files).thenReturn(files);
    when(
      () => files.uploadFromFile(
        filePath: any(named: 'filePath'),
        serverUrls: any(named: 'serverUrls'),
        contentType: any(named: 'contentType'),
      ),
    ).thenAnswer((_) => Stream.value(_completedProgress()));
    final uploader = NdkBlossomVideoUploader(
      ndk: ndk,
      servers: [BlossomServerUrl.parse('https://media.example')],
    );

    final uploaded = await uploader.upload(sampleMedia());

    expect(uploaded.primaryUrl, 'https://media.example/video.mp4');
    verify(
      () => files.uploadFromFile(
        filePath: '/tmp/ghostr-test.mp4',
        serverUrls: ['https://media.example'],
        contentType: 'video/mp4',
      ),
    ).called(1);
  });
}

BlobUploadProgress _completedProgress() {
  return BlobUploadProgress(
    currentServer: 'https://media.example',
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
          type: 'video/mp4',
        ),
      ),
    ],
  );
}
