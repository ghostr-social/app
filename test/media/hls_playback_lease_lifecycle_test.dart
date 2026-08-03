import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_lease.dart';

void main() {
  test('releases an HLS gateway session exactly once', () {
    var releases = 0;
    final lease = HlsPlaybackLease(
      ProxiedHlsVideoMediaSource(
        'http://127.0.0.1:3210/hls/'
        '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef/'
        'index.m3u8',
      ),
      () => releases += 1,
    );

    lease.release();
    lease.release();

    expect(releases, 1);
  });
}
