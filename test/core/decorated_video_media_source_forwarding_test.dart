import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('cache decorators preserve every underlying media capability', () {
    final base = VideoMediaSource.importable(
      '/picker/video.mp4',
      remoteUrl: 'https://media.test/video.mp4',
      fallbackUrls: const ['https://mirror.test/video.mp4'],
    );
    final verified = VideoMediaSource.withExpectedSha256(base, 'a' * 64);
    final scoped = VideoMediaSource.withCacheScope(verified, 'post-1');

    for (final media in [verified, scoped]) {
      expect(media.debugLabel, base.debugLabel);
      expect(media.remoteUrl, base.remoteUrl);
      expect(media.localPath, base.localPath);
      expect(media.importPath, base.importPath);
      expect(media.fallbackUrls, base.fallbackUrls);
      expect(media.isLocal, base.isLocal);
      expect(media.canCacheAsSingleFile, base.canCacheAsSingleFile);
      expect(media.remoteDelivery, base.remoteDelivery);
      expect(media.remoteUrls, base.remoteUrls);
    }
    expect(scoped.expectedSha256, verified.expectedSha256);
    expect(verified.cacheScope, isNull);
    expect(scoped.cacheScope?.value, 'post-1');
  });
}
