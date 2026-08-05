import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/live_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('native load-more uses its raw cursor and returns the newer snapshot',
      () async {
    final first = rustFeedPost(eventId: testEventId);
    final older = rustFeedPost(
      eventId: secondTestEventId,
      createdAt: 1753990000,
    );
    final port = LiveRustFeedPort(firstPage: [
      rustFeedUpdate(revision: 1, posts: [first]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    final loading = source.loadMoreRemoteFeed(searchQuery: 'ghost');
    await pumpEventQueue();
    port.publish(
      RustFeedId.parse('1'),
      rustFeedUpdate(revision: 2, posts: [first, older]),
    );

    final posts = await loading;
    expect(posts.map((post) => post.id.value), [
      testEventId,
      secondTestEventId,
    ]);
    expect(port.loadMoreCursors, [null]);
  });
}
