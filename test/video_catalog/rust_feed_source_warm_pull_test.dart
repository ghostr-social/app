import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/live_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // ndk answers a repeated pull from a session-lifetime cache merged
  // into the response stream; opening and closing a Rust feed per pull
  // made every load a cold relay round trip (up to eight seconds) for
  // rows the engine already had. One feed per spec stays open, and a
  // returning pull reads its snapshot.
  test('answers a second pull for the same spec from the live feed',
      () async {
    final port = LiveRustFeedPort(
      firstPage: [
        rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
      ],
    );
    final source = RustFeedRemoteSource(port: port);

    final first = await source.loadRemoteFeed(searchQuery: 'ghost');
    final second = await source
        .loadRemoteFeed(searchQuery: 'ghost')
        .timeout(const Duration(seconds: 2));

    expect(
      second.map((post) => post.id.value),
      first.map((post) => post.id.value),
    );
    expect(port.openedSpecs, hasLength(1), reason: 'the feed stayed open');
    expect(port.closedFeedIds, isEmpty);
  });
}
