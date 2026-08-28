import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('grace rescue starts after a no-replay watch commits', () {
    fakeAsync((clock) {
      final updates = _DeliveryUpdates();
      final history = _GatedHistory();
      final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
      final repository = FakeVideoCatalogRepository(forYouFeed: posts);
      final cubit = FeedCubit(
        FeedDependencies(
          feed: repository,
          engagement: repository,
          optional: FeedOptionalDependencies(
            delivery: FeedDeliveryDependencies(deliveryUpdates: updates),
            watch: FeedWatchDependencies(
              tracker: WatchHistoryTracker(
                history: history,
                failureReporter: RecordingFailureReporter(),
              ),
            ),
          ),
        ),
      );
      cubit.load();
      clock.flushMicrotasks();
      updates.publish(posts[0], ready: true, etaMs: 0);
      updates.publish(posts[1], ready: false, etaMs: 100);
      updates.publish(posts[2], ready: true, etaMs: 0);
      cubit.pageChanged(1);
      clock.flushMicrotasks();
      expect((cubit.state as FeedLoaded).roster.active.id.value, 'p0');

      clock.elapse(const Duration(milliseconds: 250));
      history.release.complete();
      clock.flushMicrotasks();
      expect((cubit.state as FeedLoaded).roster.active.id.value, 'p1');
      clock.elapse(const Duration(milliseconds: 250));

      expect((cubit.state as FeedLoaded).roster.active.id.value, 'p2');
      cubit.close();
      updates.close();
      clock.flushMicrotasks();
    });
  });
}

final class _GatedHistory extends FakeWatchHistoryRepository {
  final release = Completer<void>();
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    writes += 1;
    if (writes == 2) await release.future;
    await super.record(entry);
  }
}

final class _DeliveryUpdates implements VideoDeliveryUpdates {
  final _events = StreamController<VideoDeliverySnapshot>.broadcast(sync: true);

  @override
  Stream<VideoDeliverySnapshot> watchDelivery() => _events.stream;

  void publish(VideoPost post, {required bool ready, required int etaMs}) {
    _events.add(
      VideoDeliverySnapshot(
        deliveryId: post.media.playbackDeliveryId!,
        phase: ready
            ? VideoDeliveryPhase.startable
            : VideoDeliveryPhase.preparing,
        bytesPresent: BigInt.zero,
        eta: Duration(milliseconds: etaMs),
      ),
    );
  }

  Future<void> close() => _events.close();
}
