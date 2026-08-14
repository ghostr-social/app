import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('exposes the cache-scoped delivery identity before proxying', () {
    final media = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote('https://media.test/video.mp4'),
      'rust-delivery-id',
    );

    expect(
      media.playbackDeliveryId,
      PlaybackDeliveryId.parse('rust-delivery-id'),
    );
  });
}
