import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import '../../integration_test/support/warp_feed_preparation_probe.dart';

void main() {
  test('readiness timestamps stay exact to the playback authority', () {
    var elapsed = const Duration(milliseconds: 10);
    final metrics = WarpFeedPreparationMetrics(() => elapsed);
    final old = _asset('a', PlaybackPreparationReadiness.ready);
    metrics.observe(_plan(1, old));
    elapsed = const Duration(milliseconds: 20);
    final fresh = _asset('b', PlaybackPreparationReadiness.structuralStartable);
    metrics.observe(_plan(2, fresh));

    expect(
      metrics.firstAt(fresh.authority, PlaybackPreparationReadiness.ready),
      isNull,
    );
    expect(metrics.firstStructurallyStartableAt(fresh.authority), elapsed);
  });
}

PlaybackPreparationPlan _plan(int revision, PlaybackPreparationAsset asset) {
  return PlaybackPreparationPlan(
    revision: BigInt.from(revision),
    currentDeliveryId: null,
    upcoming: [asset],
  );
}

PlaybackPreparationAsset _asset(
  String identity,
  PlaybackPreparationReadiness readiness,
) {
  final digest = identity * 64;
  final capability = identity * 43;
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: PlaybackDeliveryId.parse('clip'),
      representationId: VideoRepresentationId.parse(digest),
      assetId: PlaybackAssetId.parse(capability),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:8080/video.mp4?id=clip&cap=$capability',
    ),
    readiness: readiness,
  );
}
