import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('importable media keeps its import and remote source contract', () {
    final media = VideoMediaSource.importable(
      '/picker/video.mp4',
      remoteUrl: 'https://media.test/video.mp4',
      fallbackUrls: const ['https://mirror.test/video.mp4'],
    );

    expect(media.debugLabel, 'https://media.test/video.mp4');
    expect(media.importPath, '/picker/video.mp4');
    expect(media.localPath, isNull);
    expect(media.remoteUrl, 'https://media.test/video.mp4');
    expect(media.remoteUrls, [
      'https://media.test/video.mp4',
      'https://mirror.test/video.mp4',
    ]);
    expect(media.isLocal, isFalse);
    expect(media.canCacheAsSingleFile, isTrue);
    expect(media.remoteDelivery, VideoMediaDelivery.progressive);
  });
}
