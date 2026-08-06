import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/sharing/gateway_video_file_downloader.dart';

import '../support/fake_progressive_playback_gateway.dart';

void main() {
  test('copies gateway bytes to an mp4 file for platform sharing', () async {
    final transfer = _Transfer();
    final media = VideoMediaSource.remote('https://media.test/clip.mp4');
    final gateway = FakeProgressivePlaybackGateway();
    final downloader = GatewayVideoFileDownloader(
      gateway: gateway,
      transfer: transfer,
      temporaryDirectoryPath: () async => '/tmp/ghostr',
    );

    final downloading = downloader.download(media);
    gateway.completeNext();
    final file = await downloading;

    expect(transfer.source.host, '127.0.0.1');
    expect(transfer.destination, endsWith('.mp4'));
    expect(file.path, transfer.destination);
    expect(file.path, isNot(media.remoteUrl));
  });
}

final class _Transfer implements VideoFileTransfer {
  late Uri source;
  late String destination;

  @override
  Future<void> transfer(Uri source, String destination) async {
    this.source = source;
    this.destination = destination;
  }
}
