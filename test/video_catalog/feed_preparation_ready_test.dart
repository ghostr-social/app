import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/prepared_progressive_playback.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_preparation_reducer.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

void main() {
  test('player-verified Ready remains a prepared reserve asset', () {
    final current = _media('current');
    final next = _media('next');
    final asset = _asset(next);
    final plan = PlaybackPreparationPlan(
      revision: BigInt.one,
      currentDeliveryId: current.playbackDeliveryId,
      upcoming: [asset],
    );

    final result = FeedPreparationReducer().accept(plan, current, next);

    expect(result?.next?.media, asset.media);
    expect(result?.next?.readiness, PreparedPlaybackReadiness.playerVerified);
  });
}

VideoMediaSource _media(String id) {
  return VideoMediaSource.withCacheScope(
    VideoMediaSource.remote('https://media.test/$id.mp4'),
    id,
  );
}

PlaybackPreparationAsset _asset(VideoMediaSource source) {
  const capability = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: source.playbackDeliveryId!,
      representationId: VideoRepresentationId.forMedia(source),
      assetId: PlaybackAssetId.parse(capability),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:4040/video.mp4?id=next&cap=$capability',
    ),
    readiness: PlaybackPreparationReadiness.ready,
  );
}
