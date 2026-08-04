import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  final cursor = DateTime.fromMillisecondsSinceEpoch(
    1754000000 * 1000,
    isUtc: true,
  );

  // ndk parity: an older page is the `until:` slice alone
  // (video_discovery_queries.dart sends the cursor as unix seconds).
  test('claims one older page and returns only rows at or past the cursor',
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
    final port = FakeRustFeedPort(updates: [
      rustFeedBaseline(),
      rustFeedUpdate(revision: 1, posts: [newest, boundary]),
      rustFeedUpdate(revision: 2, posts: [newest, boundary, older]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed(
      searchQuery: 'ghost',
      olderThan: cursor,
    );

    expect(port.loadMoreCursors, [BigInt.from(1754000000)]);
    expect(posts.map((post) => post.id.value), [
      secondTestEventId,
      publishedTestEventId,
    ]);
  });

  test('keeps the loaded page when the feed reports exhaustion', () async {
    final only = rustFeedPost(eventId: testEventId, createdAt: 1753990000);
    final port = FakeRustFeedPort(
      updates: [
        rustFeedBaseline(),
        rustFeedUpdate(revision: 1, posts: [only]),
      ],
      moreAvailable: false,
    );
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed(
      searchQuery: 'ghost',
      olderThan: cursor,
    );

    expect(posts.single.id.value, testEventId);
  });
}
