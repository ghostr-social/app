import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/nostr/ndk_blossom_video_uploader.dart';
import 'package:mocktail/mocktail.dart';

import '../support/ndk_mocks.dart';
import '../support/sample_data.dart';

void main() {
  test('translates plugin failures while preserving app-safe failures',
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
    ).thenAnswer((_) => Stream.error(StateError('socket failed')));
    final uploader = NdkBlossomVideoUploader(ndk: ndk, servers: const []);

    await expectLater(
      uploader.upload(sampleMedia()),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          contains('Could not upload'),
        ),
      ),
    );

    const safeFailure = AppFailure('No Blossom server accepted the video.');
    when(
      () => files.uploadFromFile(
        filePath: any(named: 'filePath'),
        serverUrls: any(named: 'serverUrls'),
        contentType: any(named: 'contentType'),
      ),
    ).thenAnswer((_) => Stream.error(safeFailure));
    await expectLater(
      uploader.upload(sampleMedia()),
      throwsA(same(safeFailure)),
    );
  });
}
