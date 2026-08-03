import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

const _session =
    '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';

void main() {
  test('accepts only the embedded gateway root manifest shape', () {
    final source = VideoMediaSource.proxiedHls(
      'http://127.0.0.1:3210/hls/$_session/index.m3u8',
    );

    expect(source, isA<ProxiedHlsVideoMediaSource>());
    expect(source.remoteDelivery, VideoMediaDelivery.hls);
    expect(source.remoteUrl, isNull);
    expect(source.remoteUrls, isEmpty);
    expect(source.fallbackUrls, isEmpty);
    expect(source.isLocal, isFalse);
    expect(source.canCacheAsSingleFile, isFalse);

    for (final invalid in [
      'https://127.0.0.1:3210/hls/$_session/index.m3u8',
      'http://localhost:3210/hls/$_session/index.m3u8',
      'http://192.168.1.2:3210/hls/$_session/index.m3u8',
      'http://127.0.0.1/hls/$_session/index.m3u8',
      'http://127.0.0.1:3210/hls/not-a-session/index.m3u8',
      'http://127.0.0.1:3210/hls/$_session/assets/1',
      'http://127.0.0.1:3210/hls/$_session/index.m3u8?token=leak',
    ]) {
      expect(() => VideoMediaSource.proxiedHls(invalid), throwsFormatException);
    }
  });
}
