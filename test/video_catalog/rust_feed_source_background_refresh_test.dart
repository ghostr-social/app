import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/live_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // Answering warm must not let the feed go stale: the pull returns
  // what is known and asks Rust for the next page behind it, so the
  // rows are there for the pull after this one. Nothing waits for it.
  test('asks the engine for another page after answering warm', () async {
    final port = LiveRustFeedPort(
      firstPage: [
        rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
      ],
    );
    final source = RustFeedRemoteSource(port: port);

    await source.loadRemoteFeed(searchQuery: 'ghost');
    expect(port.loadMoreCursors, isEmpty, reason: 'the cold pull just loaded');

    await source.loadRemoteFeed(searchQuery: 'ghost');
    await pumpEventQueue();

    expect(
      port.loadMoreCursors,
      [null],
      reason: 'the engine picks the cursor for a background page',
    );
  });
}
