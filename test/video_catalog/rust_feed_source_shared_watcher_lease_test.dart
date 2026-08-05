import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/live_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('canceling one shared watcher keeps the other watcher alive', () async {
    final port = LiveRustFeedPort();
    final source = RustFeedRemoteSource(port: port);
    final first = source.watchRemoteFeed(searchQuery: 'ghost').listen((_) {});
    final pages = <List<String>>[];
    final errors = <Object>[];
    final second = source
        .watchRemoteFeed(searchQuery: 'ghost')
        .map(
          (snapshot) => snapshot.posts.map((post) => post.id.value).toList(),
        )
        .listen(pages.add, onError: errors.add);
    addTearDown(first.cancel);
    addTearDown(second.cancel);
    await pumpEventQueue();

    expect(port.openedSpecs, hasLength(1));
    await first.cancel();
    await pumpEventQueue();

    expect(port.closedFeedIds, isEmpty);
    port.publish(
      RustFeedId.parse('1'),
      rustFeedUpdate(
        revision: 1,
        posts: [rustFeedPost(eventId: testEventId)],
      ),
    );
    await pumpEventQueue();
    expect(pages, [
      [testEventId],
    ]);
    expect(errors, isEmpty);

    await second.cancel();
    expect(port.closedFeedIds.map((feed) => feed.value), ['1']);
  });
}
