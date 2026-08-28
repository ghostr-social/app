import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('identifies each trusted loopback delivery without guessing', () {
    const hlsId =
        '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
    const capability = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
    final hls = VideoMediaSource.proxiedHls(
      'http://127.0.0.1:8080/hls/$hlsId/index.m3u8',
    );
    final progressive = ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:8080/video.mp4?id=delivery-7&cap=$capability',
    );

    expect(hls.playbackDeliveryId, isNull);
    expect(
      progressive.playbackDeliveryId,
      PlaybackDeliveryId.parse('delivery-7'),
    );
    expect(
      VideoMediaSource.local('/cache/video.mp4').playbackDeliveryId,
      isNull,
    );
    expect(() => PlaybackDeliveryId.parse('  '), throwsFormatException);
  });
}
