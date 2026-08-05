import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/live_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // Reading a snapshot is passive. Rust owns whether and when another Nostr
  // request is needed; a Dart cache read must never deepen the feed.
  test('does not request another page after answering warm', () async {
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

    expect(port.loadMoreCursors, isEmpty);
  });
}
