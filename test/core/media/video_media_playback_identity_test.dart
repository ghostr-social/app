import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('playback identity includes the concrete local or proxy endpoint', () {
    final firstCached = _cached('/cache/one.mp4');
    final secondCached = _cached('/cache/two.mp4');
    final firstProxy = _proxy('a');
    final secondProxy = _proxy('b');

    expect(firstCached.cacheJobIdentity, secondCached.cacheJobIdentity);
    expect(
      firstCached.inventoryPlaybackIdentity,
      isNot(secondCached.inventoryPlaybackIdentity),
    );
    expect(firstProxy.cacheJobIdentity, secondProxy.cacheJobIdentity);
    expect(
      firstProxy.inventoryPlaybackIdentity,
      isNot(secondProxy.inventoryPlaybackIdentity),
    );
  });
}

VideoMediaSource _cached(String path) {
  return VideoMediaSource.cached(
    path,
    remoteUrl: 'https://media.test/video.mp4',
  );
}

VideoMediaSource _proxy(String digit) {
  final session = List<String>.filled(64, digit).join();
  return VideoMediaSource.proxiedHls(
    'http://127.0.0.1:9000/hls/$session/index.m3u8',
  );
}
