import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/live_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('keeps watching an empty failed feed until Rust retry lands', () async {
    final port = LiveRustFeedPort();
    final source = RustFeedRemoteSource(port: port);
    final snapshots = <RemoteVideoSnapshot>[];
    final errors = <Object>[];
    final subscription = source
        .watchRemoteFeed(searchQuery: 'ghost')
        .listen(snapshots.add, onError: errors.add);
    await pumpEventQueue();

    final loading = source.loadRemoteFeed(searchQuery: 'ghost');
    final feed = RustFeedId.parse('1');
    port.publish(feed, rustFeedUpdate(revision: 1, stage: FfiFeedStage.failed));
    await pumpEventQueue();

    expect(errors, isEmpty);
    expect(port.openedSpecs, hasLength(1));
    expect(port.closedFeedIds, isEmpty);
    expect(snapshots.single.phase, RemoteVideoPhase.failed);

    final post = rustFeedPost();
    port.publish(
      feed,
      rustFeedUpdate(revision: 2, stage: FfiFeedStage.loading),
    );
    port.publish(feed, rustFeedUpdate(revision: 3, posts: [post]));

    expect((await loading).single.id.value, post.eventId);
    await pumpEventQueue();
    expect(snapshots.last.posts.single.id.value, post.eventId);
    expect(port.openedSpecs, hasLength(1));
    expect(port.closedFeedIds, isEmpty);
    await subscription.cancel();
  });
}
