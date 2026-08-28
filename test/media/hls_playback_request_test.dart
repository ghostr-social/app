import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';

void main() {
  test('builds a bounded gateway request only from canonical remote HLS', () {
    final hls = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote(
        'https://primary.test/root.m3u8',
        fallbackUrls: const ['https://mirror.test/root.m3u8'],
        delivery: VideoMediaDelivery.hls,
      ),
      'post-A',
    );
    final request = HlsPlaybackRequest.fromMedia(hls);

    expect(request.deliveryId.value, 'post-A');
    expect(request.sourceUrls, [
      Uri.parse('https://primary.test/root.m3u8'),
      Uri.parse('https://mirror.test/root.m3u8'),
    ]);
    expect(
      () => HlsPlaybackRequest.fromMedia(
        VideoMediaSource.remote('https://media.test/video.mp4'),
      ),
      throwsArgumentError,
    );
    expect(
      () => HlsPlaybackRequest.fromMedia(
        VideoMediaSource.proxiedHls(
          'http://127.0.0.1:3210/hls/'
          '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef/'
          'index.m3u8',
        ),
      ),
      throwsArgumentError,
    );
    expect(
      () => HlsPlaybackRequest.fromMedia(
        VideoMediaSource.remote(
          'https://user:secret@media.test/root.m3u8',
          delivery: VideoMediaDelivery.hls,
        ),
      ),
      throwsFormatException,
    );
  });
}
