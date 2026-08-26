import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import '../../integration_test/support/warp_feed_preparation_probe.dart';

void main() {
  test('latest preparation depth does not retain a historical ready state', () {
    var elapsed = const Duration(milliseconds: 10);
    final metrics = WarpFeedPreparationMetrics(() => elapsed);
    final ready = _asset('a', PlaybackPreparationReadiness.ready);
    metrics.observe(_plan(1, [ready]));
    elapsed = const Duration(milliseconds: 20);
    final structural = _asset(
      'b',
      PlaybackPreparationReadiness.structuralStartable,
    );
    metrics.observe(_plan(2, [structural]));
    elapsed = const Duration(milliseconds: 30);
    metrics.observe(_plan(3, const [], current: ready));

    expect(metrics.maximumReadyDepth, 1);
    expect(metrics.latest.revision, BigInt.from(3));
    expect(metrics.latest.elapsed, elapsed);
    expect(metrics.latest.structuralDepth, 0);
    expect(metrics.latest.readyDepth, 0);
    expect(metrics.latest.current?.authority, ready.authority);
    expect(metrics.latest.upcoming, isEmpty);
    final first = metrics.atOrBefore(const Duration(milliseconds: 15))!;
    expect(first.upcoming.single.authority, ready.authority);
    expect(first.upcoming.single.readiness.isPlayerVerified, isTrue);
    final second = metrics.atOrBefore(const Duration(milliseconds: 25))!;
    expect(second.upcoming.single.authority, structural.authority);
    expect(second.readyDepth, 0);
    expect(metrics.atOrBeforeSequence(second.sequence), same(second));
    expect(metrics.atOrBefore(const Duration(milliseconds: 5)), isNull);
  });
}

PlaybackPreparationPlan _plan(
  int revision,
  List<PlaybackPreparationAsset> upcoming, {
  PlaybackPreparationAsset? current,
}) => PlaybackPreparationPlan(
  revision: BigInt.from(revision),
  currentDeliveryId: current?.deliveryId,
  current: current,
  upcoming: upcoming,
);

PlaybackPreparationAsset _asset(
  String identity,
  PlaybackPreparationReadiness readiness,
) {
  final capability = identity * 43;
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: PlaybackDeliveryId.parse('clip-$identity'),
      representationId: VideoRepresentationId.parse(identity * 64),
      assetId: PlaybackAssetId.parse(capability),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:8080/video.mp4?id=clip-$identity&cap=$capability',
    ),
    readiness: readiness,
  );
}
