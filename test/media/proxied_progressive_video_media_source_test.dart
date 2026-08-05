import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  const trusted = 'http://127.0.0.1:3210/video.mp4?id=post_1-A';

  test('accepts a loopback progressive gateway URL', () {
    final media = ProxiedProgressiveVideoMediaSource(trusted);

    expect(media.playbackUri.toString(), trusted);
    expect(media.isLocal, isFalse);
    expect(media.localPath, isNull);
    expect(media.fallbackUrls, isEmpty);
    expect(media.canCacheAsSingleFile, isFalse);
    expect(media.remoteDelivery, VideoMediaDelivery.progressive);
    expect(media.remoteUrls, [trusted]);
    expect(media.debugLabel, 'Progressive loopback stream');
  });

  test('rejects URLs that are not the loopback progressive route', () {
    const rejected = [
      'https://127.0.0.1:3210/video.mp4?id=post-1',
      'http://media.test/video.mp4?id=post-1',
      'http://127.0.0.1/video.mp4?id=post-1',
      'http://127.0.0.1:3210/video.mp4',
      'http://127.0.0.1:3210/other.mp4?id=post-1',
      'http://127.0.0.1:3210/video.mp4?id=post-1&extra=1',
      'http://127.0.0.1:3210/video.mp4?id=bad%20id',
      'http://127.0.0.1:3210/video.mp4?id=',
      'http://127.0.0.1:3210/video.mp4?id=post-1#clip',
      'http://user@127.0.0.1:3210/video.mp4?id=post-1',
    ];

    for (final url in rejected) {
      expect(
        () => ProxiedProgressiveVideoMediaSource(url),
        throwsFormatException,
        reason: url,
      );
    }
  });
}
