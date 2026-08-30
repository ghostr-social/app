import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import 'fakes.dart';
import 'fake_progressive_playback_gateway.dart';
import 'feed_preparation_updates.dart';
import 'feed_preparation_video_player_platform.dart';
import 'feed_screen_harness.dart';
import 'sample_data.dart';
part 'feed_preparation_fixture_assets.dart';

final class FeedPreparationFixture {
  FeedPreparationFixture({int postCount = 3})
    : posts = List.generate(postCount, _preparationPost);
  final platform = FeedPreparationVideoPlayerPlatform();
  final updates = ControlledPlaybackPreparationUpdates();
  final List<VideoPost> posts;
  Future<void> pump(
    WidgetTester tester, {
    VideoPlaybackPort? playbackPort,
    FeedFocusPort? focus,
  }) async {
    VideoPlayerPlatform.instance = platform;
    final repository = FakeVideoCatalogRepository(forYouFeed: posts);
    final tracker = WatchHistoryTracker(
      history: FakeWatchHistoryRepository(),
      failureReporter: RecordingFailureReporter(),
    );
    final playback = playbackPort ?? _defaultPlayback();
    await tester.pumpWidget(
      feedScreenHarness(
        repository,
        options: FeedScreenHarnessOptions(
          playbackPort: playback,
          focus: focus,
          preparationUpdates: updates,
          watch: FeedWatchDependencies(tracker: tracker),
        ),
      ),
    );
    await tester.pumpAndSettle();
  }

  VideoPlaybackPort _defaultPlayback() {
    return GatewayVideoPlaybackPort(
      delegate: VideoPlayerPlaybackPort(),
      gateway: FakeProgressivePlaybackGateway(immediatePlaybackUrl: url('p0')),
    );
  }

  void publish(int revision, String current, String? next) {
    publishWindow(revision, current, next == null ? const [] : [next]);
  }

  void publishWindow(int revision, String current, List<String> upcoming) {
    updates.publish(
      PlaybackPreparationPlan(
        revision: BigInt.from(revision),
        currentDeliveryId: PlaybackDeliveryId.parse(current),
        current: _preparationAsset(posts, current),
        upcoming: upcoming
            .map((id) => _preparationAsset(posts, id))
            .toList(growable: false),
      ),
    );
  }

  Future<void> swipe(WidgetTester tester) async {
    final page = find.byType(PageView);
    final height = tester.getSize(page).height;
    final gesture = await tester.startGesture(tester.getCenter(page));
    await gesture.moveBy(Offset(0, -height * 0.23));
    await tester.pump(const Duration(milliseconds: 16));
    await gesture.up();
    await tester.pumpAndSettle();
    await settle(tester);
  }

  String url(String id) =>
      _preparationAsset(posts, id).media.playbackUri.toString();

  Future<void> settle(WidgetTester tester) async {
    await tester.pump();
    await tester.pumpAndSettle(const Duration(milliseconds: 20));
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pump();
  }
}
