import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('models an import candidate without making it directly playable', () {
    final media = VideoMediaSource.importable(
      ' /native/video.mp4 ',
      remoteUrl: 'https://media.example/video.mp4',
      fallbackUrls: ['https://mirror.example/video.mp4'],
    );

    expect(media.importPath, '/native/video.mp4');
    expect(media.localPath, isNull);
    expect(media.isLocal, isFalse);
    expect(media.canCacheAsSingleFile, isTrue);
    expect(media.remoteUrls, [
      'https://media.example/video.mp4',
      'https://mirror.example/video.mp4',
    ]);
    expect(media.remoteDelivery, VideoMediaDelivery.progressive);
  });
}
