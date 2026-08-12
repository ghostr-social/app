import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ghostr/platform/nostr/ndk_blossom_profile_image_uploader.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/fake_profile_image_services.dart';
import '../support/ndk_mocks.dart';

void main() {
  test('rejects failed, malformed, and non-image Blossom responses', () async {
    final ndk = MockNdk();
    final files = MockFiles();
    when(() => ndk.files).thenReturn(files);
    final uploader = NdkBlossomProfileImageUploader(
      ndk: ndk,
      servers: [BlossomServerUrl.parse('https://media.example')],
    );

    when(
      () => files.uploadFromFile(
        filePath: any(named: 'filePath'),
        serverUrls: any(named: 'serverUrls'),
        contentType: any(named: 'contentType'),
      ),
    ).thenAnswer((_) => Stream.value(_progress(type: 'video/mp4')));
    await expectLater(
      uploader.upload(sampleProfileImage()),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          contains('accepted'),
        ),
      ),
    );

    when(
      () => files.uploadFromFile(
        filePath: any(named: 'filePath'),
        serverUrls: any(named: 'serverUrls'),
        contentType: any(named: 'contentType'),
      ),
    ).thenThrow(StateError('plugin crashed'));
    await expectLater(
      uploader.upload(sampleProfileImage()),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          contains('Could not upload'),
        ),
      ),
    );
  });
}

BlobUploadProgress _progress({required String type}) => BlobUploadProgress(
  currentServer: 'https://media.example',
  sentBytes: 10,
  totalBytes: 10,
  isComplete: true,
  completedUploads: [
    BlobUploadResult(
      serverUrl: 'https://media.example',
      success: true,
      descriptor: BlobDescriptor(
        url: 'https://media.example/not-an-image.mp4',
        sha256: 'a' * 64,
        size: 10,
        uploaded: DateTime(2026, 8, 11),
        type: type,
      ),
    ),
  ],
);
