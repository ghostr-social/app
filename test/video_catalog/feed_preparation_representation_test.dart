import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_preparation_reducer.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

void main() {
  test('same-post media revision cannot consume the prior exact asset', () {
    final oldMedia = _media('old');
    final newMedia = _media('new');
    final reducer = FeedPreparationReducer();
    final result = reducer.accept(_plan(_asset(oldMedia)), newMedia, null);

    expect(oldMedia.playbackDeliveryId, newMedia.playbackDeliveryId);
    expect(result?.isManaged, isTrue);
    expect(result?.current, isNull);
  });
}

VideoMediaSource _media(String revision) {
  final remote = VideoMediaSource.remote('https://media.test/$revision.mp4');
  return VideoMediaSource.withCacheScope(remote, 'same');
}

PlaybackPreparationPlan _plan(PlaybackPreparationAsset asset) {
  return PlaybackPreparationPlan(
    revision: BigInt.one,
    currentDeliveryId: PlaybackDeliveryId.parse('same'),
    current: asset,
  );
}

PlaybackPreparationAsset _asset(VideoMediaSource media) {
  const capability = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: PlaybackDeliveryId.parse('same'),
      representationId: VideoRepresentationId.forMedia(media),
      assetId: PlaybackAssetId.parse(capability),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:4040/video.mp4?id=same&cap=$capability',
    ),
    readiness: PlaybackPreparationReadiness.structuralStartable,
  );
}
