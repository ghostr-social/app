import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('cache identities encode digest scope sources and import path', () {
    final base = VideoMediaSource.importable(
      '/picker/video.mp4',
      remoteUrl: 'https://media.test/video.mp4',
    );
    final scoped = VideoMediaSource.withCacheScope(base, 'post-1');
    final verified = VideoMediaSource.withExpectedSha256(scoped, 'a' * 64);

    expect(base.cacheStorageIdentity.value, contains('scope:8:unscoped'));
    expect(scoped.cacheStorageIdentity.value, contains('scope:6:post-1'));
    expect(verified.cacheStorageIdentity.value, 'sha256:${'a' * 64}');
    expect(
        base.cacheJobIdentity.value, contains('import:17:/picker/video.mp4'));
    expect(base.cacheStorageIdentity, base.cacheStorageIdentity);
    expect(
      base.cacheStorageIdentity.hashCode,
      base.cacheStorageIdentity.hashCode,
    );
  });
}
