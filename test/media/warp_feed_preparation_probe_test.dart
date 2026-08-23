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
      metrics.firstAt(
        authority,
        PlaybackPreparationReadiness.structuralStartable,
      ),
      elapsed,
    );
  });

  test('Ready depth is exact while structural depth is inclusive', () {
    const elapsed = Duration(milliseconds: 40);
    final metrics = WarpFeedPreparationMetrics(() => elapsed);
    final structural = _asset(
      'structural',
      'b',
      PlaybackPreparationReadiness.structuralStartable,
    );
    final ready = _asset('ready', 'c', PlaybackPreparationReadiness.ready);

    metrics.observe(
      PlaybackPreparationPlan(
        revision: BigInt.one,
        currentDeliveryId: null,
        upcoming: [structural, ready],
      ),
    );

    expect(metrics.maximumStructuralDepth, 2);
    expect(metrics.maximumReadyDepth, 1);
    expect(
      metrics.firstAt(ready.authority, PlaybackPreparationReadiness.ready),
      elapsed,
    );
    expect(metrics.firstStructurallyStartableAt(ready.authority), elapsed);
  });
}

PlaybackAssetAuthority _authority() => PlaybackAssetAuthority(
  deliveryId: PlaybackDeliveryId.parse('delivery'),
  representationId: VideoRepresentationId.parse(
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
  ),
  assetId: PlaybackAssetId.parse('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'),
);

PlaybackPreparationAsset _asset(
  String id,
  String identity,
  PlaybackPreparationReadiness readiness,
) {
  final digest = identity * 64;
  final capability = identity * 43;
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: PlaybackDeliveryId.parse(id),
      representationId: VideoRepresentationId.parse(digest),
      assetId: PlaybackAssetId.parse(capability),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:8080/video.mp4?id=$id&cap=$capability',
    ),
    readiness: readiness,
  );
}
