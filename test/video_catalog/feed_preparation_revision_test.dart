import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_preparation_reducer.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

void main() {
  test('equal revisions replace authoritatively and older plans are stale', () {
    final reducer = FeedPreparationReducer();
    final media = _media('p0');
    final first = reducer.accept(_plan(2, _asset('p0', media)), media, null);
    expect(first?.current, isNotNull);

    final removed = reducer.accept(_plan(2, null), media, null);
    expect(removed?.isManaged, isTrue);
    expect(removed?.current, isNull);

    final stale = reducer.accept(_plan(1, _asset('p0', media)), media, null);
    expect(stale, isNull);
    expect(reducer.watermark, BigInt.from(2));
  });
}

PlaybackPreparationPlan _plan(int revision, PlaybackPreparationAsset? asset) {
  return PlaybackPreparationPlan(
    revision: BigInt.from(revision),
    currentDeliveryId: PlaybackDeliveryId.parse('p0'),
    current: asset,
  );
}

VideoMediaSource _media(String id) {
  final remote = VideoMediaSource.remote('https://media.test/$id.mp4');
  return VideoMediaSource.withCacheScope(remote, id);
}

PlaybackPreparationAsset _asset(String id, VideoMediaSource source) {
  const capability = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: PlaybackDeliveryId.parse(id),
      representationId: VideoRepresentationId.forMedia(source),
      assetId: PlaybackAssetId.parse(capability),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:4040/video.mp4?id=$id&cap=$capability',
    ),
    readiness: PlaybackPreparationReadiness.structuralStartable,
  );
}
