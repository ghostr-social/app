import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import '../support/fake_dependencies.dart';
import '../support/fake_video_catalog_repository.dart';
import '../support/feed_preparation_updates.dart';
import '../support/sample_data.dart';

void main() {
  test('main feed consumes the app preparation dependency', () async {
    final updates = ControlledPlaybackPreparationUpdates();
    addTearDown(updates.close);
    final media = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote('https://media.test/p0.mp4'),
      'p0',
    );
    final factory = AppControllerFactory(
      buildFakeDependencies(
        catalogRepository: FakeVideoCatalogRepository(
          forYouFeed: [samplePost(id: 'p0').withMedia(media)],
        ),
        overrides: FakeDependencyOverrides(preparationUpdates: updates),
      ),
    );
    final feed = factory.feed();
    addTearDown(feed.close);
    await feed.load();

    updates.publish(_plan(media));
    await Future<void>.delayed(Duration.zero);

    final loaded = feed.state as FeedLoaded;
    expect(loaded.preparation.current?.media.remoteUrl, _playbackUrl);
  });
}

PlaybackPreparationPlan _plan(VideoMediaSource source) {
  final authority = PlaybackAssetAuthority(
    deliveryId: PlaybackDeliveryId.parse('p0'),
    representationId: VideoRepresentationId.forMedia(source),
    assetId: PlaybackAssetId.parse(_capability),
  );
  return PlaybackPreparationPlan(
    revision: BigInt.one,
    currentDeliveryId: authority.deliveryId,
    current: PlaybackPreparationAsset(
      authority: authority,
      media: ProxiedProgressiveVideoMediaSource(_playbackUrl),
      readiness: PlaybackPreparationReadiness.structuralStartable,
    ),
  );
}

const _capability = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _playbackUrl = 'http://127.0.0.1:4040/video.mp4?id=p0&cap=$_capability';
