import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'a delivery rescue changes focus without recording a user watch',
    () async {
      final updates = _DeliveryUpdates();
      final focus = FakeFeedFocusPort();
      final history = FakeWatchHistoryRepository();
      final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
      final repository = FakeVideoCatalogRepository(forYouFeed: posts);
      final cubit = FeedCubit(
        FeedDependencies(
          feed: repository,
          engagement: repository,
          optional: FeedOptionalDependencies(
            focus: focus,
            deliveryUpdates: updates,
            watchTracker: WatchHistoryTracker(
              history: history,
              settings: FakeAppSettingsRepository(AppSettings.defaults()),
              failureReporter: RecordingFailureReporter(),
            ),
          ),
        ),
      );
      addTearDown(cubit.close);
      addTearDown(updates.close);
      await cubit.load();
      updates.publish(posts[1], startable: false);
      updates.publish(posts[2], startable: true);

      cubit.pageChanged(1);
      await pumpEventQueue();

      expect((cubit.state as FeedLoaded).activeIndex, 2);
      expect(focus.focuses.last.cause, FeedFocusCause.transportRescue);
      expect(history.entries.map((entry) => entry.videoId), ['e:p0']);
    },
  );
}

final class _DeliveryUpdates implements VideoDeliveryUpdates {
  final _events = StreamController<VideoDeliverySnapshot>.broadcast(sync: true);

  @override
  Stream<VideoDeliverySnapshot> watchDelivery() => _events.stream;

  void publish(VideoPost post, {required bool startable}) {
    final deliveryId = post.media.playbackDeliveryId!;
    _events.add(
      VideoDeliverySnapshot(
        deliveryId: deliveryId,
        phase: startable
            ? VideoDeliveryPhase.startable
            : VideoDeliveryPhase.preparing,
        bytesPresent: BigInt.zero,
      ),
    );
  }

  Future<void> close() => _events.close();
}
