import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_preparation_reducer.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

void main() {
  test('a progressive preparation plan does not manage HLS playback', () {
    final remote = VideoMediaSource.remote(
      'https://media.test/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    );
    final media = VideoMediaSource.withCacheScope(remote, 'current');
    final plan = PlaybackPreparationPlan(
      revision: BigInt.one,
      currentDeliveryId: PlaybackDeliveryId.parse('current'),
    );

    final result = FeedPreparationReducer().accept(plan, media, null);

    expect(result?.isManaged, isFalse);
    expect(result?.current, isNull);
  });
}
