import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import '../../integration_test/support/warp_feed_preparation_probe.dart';

void main() {
  test('preparation metrics timestamp current structural readiness', () {
    var elapsed = const Duration(milliseconds: 25);
    final metrics = WarpFeedPreparationMetrics(() => elapsed);
    final authority = _authority();
    metrics.observe(
      PlaybackPreparationPlan(
        revision: BigInt.one,
        currentDeliveryId: authority.deliveryId,
        current: PlaybackPreparationAsset(
          authority: authority,
          media: ProxiedProgressiveVideoMediaSource(
            'http://127.0.0.1:8080/video.mp4?id=delivery&cap='
            'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
          ),
          readiness: PlaybackPreparationReadiness.structuralStartable,
        ),
      ),
    );

    expect(
      metrics.firstCurrentAt(
        authority.deliveryId,
        PlaybackPreparationReadiness.structuralStartable,
      ),
      elapsed,
    );
  });
}

PlaybackAssetAuthority _authority() => PlaybackAssetAuthority(
  deliveryId: PlaybackDeliveryId.parse('delivery'),
  representationId: VideoRepresentationId.parse(
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
  ),
  assetId: PlaybackAssetId.parse('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'),
);
