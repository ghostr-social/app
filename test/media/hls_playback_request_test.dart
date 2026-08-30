import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
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
    final representation = VideoRepresentationId.forMedia(hls);
    final authority = HlsPlaybackAuthority(
      deliveryId: PlaybackDeliveryId.parse('post-A'),
      representationId: representation,
      assetRevision: HlsPlaybackAssetRevision.parse(BigInt.from(7)),
    );
    final request = HlsPlaybackRequest.fromMedia(
      hls,
      expectedAuthority: authority,
    );

    expect(request.deliveryId.value, 'post-A');
    expect(request.representationId, representation);
    expect(request.expectedAuthority, authority);
    expect(request.sourceUrls, [
      Uri.parse('https://primary.test/root.m3u8'),
      Uri.parse('https://mirror.test/root.m3u8'),
    ]);
    expect(HlsPlaybackRequest.fromMedia(hls).expectedAuthority, isNull);
    for (final mismatched in [
      HlsPlaybackAuthority(
        deliveryId: PlaybackDeliveryId.parse('post-B'),
        representationId: representation,
        assetRevision: authority.assetRevision,
      ),
      HlsPlaybackAuthority(
        deliveryId: authority.deliveryId,
        representationId: VideoRepresentationId.parse('b' * 64),
        assetRevision: authority.assetRevision,
      ),
    ]) {
      expect(
        () => HlsPlaybackRequest.fromMedia(hls, expectedAuthority: mismatched),
        throwsArgumentError,
      );
    }
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
