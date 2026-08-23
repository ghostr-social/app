import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_preparation_reducer.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

void main() {
  test('derived playback authority binds to its original feed source', () {
    final source = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote('https://media.test/original.mp4'),
      'clip',
    );
    const capability = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
    final asset = PlaybackPreparationAsset(
      authority: PlaybackAssetAuthority(
        deliveryId: source.playbackDeliveryId!,
        representationId: VideoRepresentationId.parse('b' * 64),
        assetId: PlaybackAssetId.parse(capability),
      ),
      sourceRepresentationId: VideoRepresentationId.forMedia(source),
      media: ProxiedProgressiveVideoMediaSource(
        'http://127.0.0.1:4040/video.mp4?id=clip&cap=$capability',
      ),
      readiness: PlaybackPreparationReadiness.structuralStartable,
    );

    final current = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote('https://media.test/current.mp4'),
      'current',
    );
    final plan = PlaybackPreparationPlan(
      revision: BigInt.one,
      currentDeliveryId: current.playbackDeliveryId,
      upcoming: [asset],
    );

    final preparation = FeedPreparationReducer().accept(plan, current, source);

    expect(preparation?.next?.authority, asset.authority);
    expect(preparation?.next?.matches(source), isTrue);
  });
}
