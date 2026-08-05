import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/live_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('passively streams every useful Rust feed revision', () async {
    final port = LiveRustFeedPort();
    final source = RustFeedRemoteSource(port: port);
    final pages = <List<String>>[];
    final subscription = source
        .watchRemoteFeed(searchQuery: 'ghost')
        .map(
          (snapshot) => snapshot.posts.map((post) => post.id.value).toList(),
        )
        .listen(pages.add);
    await pumpEventQueue();

    final feed = RustFeedId.parse('1');
    port.publish(feed, rustFeedBaseline());
    port.publish(
      feed,
      rustFeedUpdate(
        revision: 1,
        posts: [rustFeedPost(eventId: testEventId)],
      ),
    );
    port.publish(
      feed,
      rustFeedUpdate(revision: 2, posts: [
        rustFeedPost(eventId: testEventId),
        rustFeedPost(eventId: secondTestEventId),
      ]),
    );
    await pumpEventQueue();

    expect(pages, [
      [testEventId],
      [testEventId, secondTestEventId],
    ]);
    expect(port.loadMoreCursors, isEmpty);
    await subscription.cancel().timeout(
          const Duration(seconds: 2),
          onTimeout: () =>
              fail('watch cancel stalled; closed=${port.closedFeedIds}'),
        );
  });
}
