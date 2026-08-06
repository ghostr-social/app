import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/sharing/gateway_video_file_downloader.dart';

import '../support/fake_progressive_playback_gateway.dart';

void main() {
  test('rejects gateway media without a remote download identity', () async {
    final gateway = FakeProgressivePlaybackGateway();
    final downloader = GatewayVideoFileDownloader(
      gateway: gateway,
      transfer: _UnusedTransfer(),
      temporaryDirectoryPath: () async => '/tmp/ghostr',
    );

    final downloading = downloader.download(
      VideoMediaSource.local('/clip.mp4'),
    );
    gateway.completeNext();

    await expectLater(
      downloading,
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Could not download this video.',
        ),
      ),
    );
  });
}

final class _UnusedTransfer implements VideoFileTransfer {
  @override
  Future<void> transfer(Uri source, String destination) async {
    fail('Transfer must not start.');
  }
}
