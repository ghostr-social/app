import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';
import 'package:ghostr/platform/sharing/share_plus_video_file_port.dart';

void main() {
  test('translates a platform share-sheet failure', () async {
    final port = SharePlusVideoFilePort(
      platformShare: (_) async {
        throw StateError('share sheet unavailable');
      },
    );

    await expectLater(
      port.share(
        ShareableVideoFile.parse('/tmp/downloaded.mp4'),
        origin: VideoShareOrigin.zero,
      ),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Could not open video sharing.',
        ),
      ),
    );
  });
}
