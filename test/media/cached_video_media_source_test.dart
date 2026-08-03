import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('models cached media with local-first remote fallbacks', () {
    final media = VideoMediaSource.cached(
      ' /cache/video.mp4 ',
      remoteUrl: 'https://media.example/video.mp4',
      fallbackUrls: ['https://mirror.example/video.mp4'],
      delivery: VideoMediaDelivery.hls,
    );

    expect(media.debugLabel, '/cache/video.mp4');
    expect(media.localPath, '/cache/video.mp4');
    expect(media.remoteUrl, 'https://media.example/video.mp4');
    expect(media.fallbackUrls, ['https://mirror.example/video.mp4']);
    expect(media.remoteUrls, [
      'https://media.example/video.mp4',
      'https://mirror.example/video.mp4',
    ]);
    expect(media.remoteDelivery, VideoMediaDelivery.hls);
    expect(media.isLocal, isTrue);
    expect(media.canCacheAsSingleFile, isFalse);
    expect(
      () => VideoMediaSource.cached(
        ' ',
        remoteUrl: 'https://media.example/video.mp4',
      ),
      throwsFormatException,
    );
  });
}
