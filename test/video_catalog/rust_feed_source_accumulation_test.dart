import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/live_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // The engine keeps filing pages into an open feed — background
  // prefetch and hunger widening both land there — so what a returning
  // pull serves is everything gathered this session, not one page.
  test('serves the rows the live feed gathered since the last pull', () async {
    final opening = rustFeedPost(eventId: testEventId, createdAt: 1754005000);
    final later = rustFeedPost(
      eventId: secondTestEventId,
      createdAt: 1754000000,
    );
    final port = LiveRustFeedPort(
      firstPage: [
        rustFeedUpdate(revision: 1, posts: [opening]),
      ],
    );
    final source = RustFeedRemoteSource(port: port);

    final first = await source.loadRemoteFeed(searchQuery: 'ghost');
    port.publish(
      RustFeedId.parse('1'),
      rustFeedUpdate(revision: 2, posts: [opening, later]),
    );
    await pumpEventQueue();
    final second = await source.loadRemoteFeed(searchQuery: 'ghost');

    expect(first.map((post) => post.id.value), [testEventId]);
    expect(
      second.map((post) => post.id.value),
      [testEventId, secondTestEventId],
      reason: 'a returning pull accumulates, it never replaces',
    );
  });
}
