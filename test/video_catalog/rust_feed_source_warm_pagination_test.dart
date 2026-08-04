import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/live_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  final cursor = DateTime.fromMillisecondsSinceEpoch(
    1754000000 * 1000,
    isUtc: true,
  );

  // Older pages the engine prefetched land in the same open feed, so a
  // swipe past the cursor is answered from what is already known —
  // only a cursor past everything the feed holds waits for relays.
  test('answers an older page from rows the live feed already holds',
      () async {
    final newest = rustFeedPost(eventId: testEventId, createdAt: 1754005000);
    final boundary = rustFeedPost(
      eventId: secondTestEventId,
      createdAt: 1754000000,
    );
    final older = rustFeedPost(
      eventId: publishedTestEventId,
      createdAt: 1753990000,
    );
    final port = LiveRustFeedPort(
      firstPage: [
        rustFeedUpdate(revision: 1, posts: [newest]),
      ],
    );
    final source = RustFeedRemoteSource(port: port);

    await source.loadRemoteFeed(searchQuery: 'ghost');
    port.publish(
      '1',
      rustFeedUpdate(revision: 2, posts: [newest, boundary, older]),
    );
    await pumpEventQueue();
    final page = await source.loadRemoteFeed(
      searchQuery: 'ghost',
      olderThan: cursor,
    );

    expect(
      page.map((post) => post.id.value),
      [secondTestEventId, publishedTestEventId],
    );
    expect(
      port.loadMoreCursors,
      everyElement(isNull),
      reason: 'no relay round trip is claimed for rows already held',
    );
  });
}
