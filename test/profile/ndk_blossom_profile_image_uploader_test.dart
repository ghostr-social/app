import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ghostr/platform/nostr/ndk_blossom_profile_image_uploader.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/fake_profile_image_services.dart';
import '../support/ndk_mocks.dart';

void main() {
  test('uploads a selected image to configured Blossom servers', () async {
    final ndk = MockNdk();
    final files = MockFiles();
    when(() => ndk.files).thenReturn(files);
    when(
      () => files.uploadFromFile(
        filePath: any(named: 'filePath'),
        serverUrls: any(named: 'serverUrls'),
        contentType: any(named: 'contentType'),
      ),
    ).thenAnswer((_) => Stream.value(_progress()));
    final uploader = NdkBlossomProfileImageUploader(
      ndk: ndk,
      servers: [BlossomServerUrl.parse('https://media.example')],
    );

    final uploaded = await uploader.upload(sampleProfileImage());

    expect(uploaded.value, 'https://media.example/avatar.png');
    verify(
      () => files.uploadFromFile(
        filePath: '/tmp/nora-avatar.png',
        serverUrls: ['https://media.example'],
        contentType: 'image/png',
      ),
    ).called(1);
  });
}

BlobUploadProgress _progress() => BlobUploadProgress(
  currentServer: 'https://media.example',
  sentBytes: 10,
  totalBytes: 10,
  isComplete: true,
  completedUploads: [
    BlobUploadResult(
      serverUrl: 'https://media.example',
      success: true,
      descriptor: BlobDescriptor(
        url: 'https://media.example/avatar.png',
        sha256: 'a' * 64,
        size: 10,
        uploaded: DateTime(2026, 8, 11),
        type: 'image/png',
      ),
    ),
  ],
);
