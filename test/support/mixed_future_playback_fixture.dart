import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import 'controlled_video_delivery_updates.dart';
import 'fake_media_ports.dart';
import 'fake_video_catalog_repository.dart';
import 'feed_preparation_updates.dart';
import 'feed_screen_harness.dart';
import 'sample_data.dart';

part 'mixed_future_playback_fixture_assets.dart';

final class MixedFuturePlaybackFixture {
  final posts = _mixedPosts();
  final delivery = ControlledVideoDeliveryUpdates();
  final preparation = ControlledPlaybackPreparationUpdates();
  final playback = FakeVideoPlaybackPort();

  Future<void> prepare(WidgetTester tester, List<String> deep) async {
    await tester.pumpWidget(
      feedScreenHarness(
        FakeVideoCatalogRepository(forYouFeed: posts),
        options: FeedScreenHarnessOptions(
          playbackPort: playback,
          deliveryUpdates: delivery,
          preparationUpdates: preparation,
        ),
      ),
    );
    await tester.pumpAndSettle();
    preparation.publish(
      PlaybackPreparationPlan(
        revision: BigInt.one,
        currentDeliveryId: PlaybackDeliveryId.parse('p0'),
        current: _mixedPreparationAsset(posts, 'p0'),
        upcoming: ['p1', 'p2', 'p3', ...deep]
            .map((id) => _mixedPreparationAsset(posts, id))
            .toList(growable: false),
      ),
    );
    await _settle(tester);
    for (var index = 0; index < 3; index++) {
      await _swipe(tester);
    }
    for (final post in posts.where(_isHlsPost)) {
      delivery.publish(
        post,
        phase: VideoDeliveryPhase.startable,
        hlsAuthority: _mixedHlsAuthority(post),
      );
    }
    await _settle(tester);
  }

  Future<void> close() async {
    await delivery.close();
    await preparation.close();
  }
}

Future<void> _swipe(WidgetTester tester) async {
  final page = find.byType(PageView);
  final height = tester.getSize(page).height;
  final gesture = await tester.startGesture(tester.getCenter(page));
  await gesture.moveBy(Offset(0, -height * 0.23));
  await tester.pump(const Duration(milliseconds: 16));
  await gesture.up();
  await tester.pumpAndSettle();
  await _settle(tester);
}

Future<void> _settle(WidgetTester tester) async {
  await tester.pump();
  await tester.pumpAndSettle(const Duration(milliseconds: 20));
  await tester.runAsync(() => Future<void>.delayed(Duration.zero));
  await tester.pump();
}
