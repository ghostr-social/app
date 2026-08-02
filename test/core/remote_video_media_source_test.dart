import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('reports the primary and fallback remote video locations in order', () {
    final source = VideoMediaSource.remote(
      ' https://example.com/video.mp4 ',
      fallbackUrls: ['https://backup.example.com/video.mp4'],
    );

    expect(source.isLocal, isFalse);
    expect(source.canCacheAsSingleFile, isTrue);
    expect(source.remoteDelivery, VideoMediaDelivery.progressive);
    expect(source.localPath, isNull);
    expect(source.remoteUrl, 'https://example.com/video.mp4');
    expect(source.remoteUrls, [
      'https://example.com/video.mp4',
      'https://backup.example.com/video.mp4',
    ]);
    expect(source.fallbackUrls, ['https://backup.example.com/video.mp4']);
    expect(source.debugLabel, 'https://example.com/video.mp4');
  });
}
