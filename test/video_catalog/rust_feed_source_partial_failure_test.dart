import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/live_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('keeps partial rows live while Rust retries a failed hunt', () async {
    final port = LiveRustFeedPort();
    final source = RustFeedRemoteSource(port: port);
    final snapshots = <RemoteVideoSnapshot>[];
    final errors = <Object>[];
    final subscription = source
        .watchRemoteFeed(searchQuery: 'ghost')
        .listen(snapshots.add, onError: errors.add);
    await pumpEventQueue();

    final feed = RustFeedId.parse('1');
    final partial = [rustFeedPost(eventId: testEventId)];
    port.publish(feed, rustFeedUpdate(revision: 1, posts: partial));
    port.publish(
      feed,
      rustFeedUpdate(
        revision: 2,
        stage: FfiFeedStage.failed,
        posts: partial,
      ),
    );
    port.publish(
      feed,
      rustFeedUpdate(
        revision: 3,
        posts: [rustFeedPost(eventId: secondTestEventId)],
      ),
    );
    await pumpEventQueue();

    expect(snapshots.map((snapshot) => snapshot.phase), [
      RemoteVideoPhase.settled,
      RemoteVideoPhase.failed,
      RemoteVideoPhase.settled,
    ]);
    expect(snapshots.last.posts.single.id.value, secondTestEventId);
    expect(errors, isEmpty);
    await subscription.cancel();
  });
}
