import 'dart:async';

import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

/// One scripted snapshot and how long after subscription the Rust
/// engine publishes it.
typedef TimedFeedUpdate = ({Duration at, FfiFeedUpdate update});

/// A Rust feed port that publishes snapshots on a schedule, so tests
/// can drive the adapter's page deadline with fake time.
class TimedRustFeedPort implements RustFeedPort {
  TimedRustFeedPort(this.schedule);

  final List<TimedFeedUpdate> schedule;
  final List<String> closedFeedIds = <String>[];
  bool moreAvailable = false;
  String feedId = '7';

  @override
  Future<String> openFeed(FfiFeedSpec spec) async => feedId;

  @override
  Stream<FfiFeedUpdate> feedUpdates(String feedId) {
    final timers = <Timer>[];
    final controller = StreamController<FfiFeedUpdate>();
    controller
      ..onListen = () {
        for (final entry in schedule) {
          timers.add(Timer(entry.at, () => controller.add(entry.update)));
        }
      }
      ..onCancel = () {
        for (final timer in timers) {
          timer.cancel();
        }
      };
    return controller.stream;
  }

  @override
  Future<bool> loadMore(String feedId, {BigInt? olderThanSecs}) async =>
      moreAvailable;

  @override
  Future<void> closeFeed(String feedId) async => closedFeedIds.add(feedId);
}
