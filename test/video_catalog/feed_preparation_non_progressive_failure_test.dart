import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_video_catalog_repository.dart';
import '../support/feed_preparation_updates.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('a failed watcher releases next preparation beside HLS', (
    tester,
  ) async {
    final scenario = _scenario();
    final updates = ControlledPlaybackPreparationUpdates();
    addTearDown(updates.close);
    await tester.pumpWidget(
      feedScreenHarness(
        FakeVideoCatalogRepository(
          forYouFeed: [scenario.current, scenario.next],
        ),
        options: FeedScreenHarnessOptions(
          playbackPort: FakeVideoPlaybackPort(),
          preparationUpdates: updates,
        ),
      ),
    );
    await tester.pumpAndSettle();
    updates.publish(_plan(scenario.asset));
    await tester.pumpAndSettle();
    final loaded = tester
        .element(find.byType(PageView))
        .read<FeedCubit>()
        .state;
    expect((loaded as FeedLoaded).preparation.next, isNotNull);
    expect(
      find.text(scenario.asset.media.debugLabel, skipOffstage: false),
      findsOneWidget,
    );

    updates.fail(StateError('native watcher failed'));
    await tester.pumpAndSettle();

    expect(find.text(scenario.current.media.debugLabel), findsOneWidget);
    expect(
      find.text(scenario.asset.media.debugLabel, skipOffstage: false),
      findsNothing,
    );
  });
}

({VideoPost current, VideoPost next, PlaybackPreparationAsset asset})
_scenario() {
  final current = samplePost(id: 'hls').withMedia(
    VideoMediaSource.remote(
      'https://media.test/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    ),
  );
  final media = VideoMediaSource.withCacheScope(
    VideoMediaSource.remote('https://media.test/next.mp4'),
    'next',
  );
  final next = samplePost(id: 'next').withMedia(media);
  return (current: current, next: next, asset: _asset(media));
}

PlaybackPreparationPlan _plan(PlaybackPreparationAsset asset) {
  return PlaybackPreparationPlan(
    revision: BigInt.one,
    currentDeliveryId: asset.deliveryId,
    current: asset,
  );
}

PlaybackPreparationAsset _asset(VideoMediaSource media) {
  const capability = 'nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn';
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: PlaybackDeliveryId.parse('next'),
      representationId: VideoRepresentationId.forMedia(media),
      assetId: PlaybackAssetId.parse(capability),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:4040/video.mp4?id=next&cap=$capability',
    ),
    readiness: PlaybackPreparationReadiness.structuralStartable,
  );
}
