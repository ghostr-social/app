import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_sessions.dart';

import '../support/live_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('an observed search is not evicted by unrelated feed pulls', () async {
    final port = LiveRustFeedPort(
      firstPage: [
        rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
      ],
    );
    final source = RustFeedRemoteSource(port: port);
    final watch = source.watchRemoteFeed(searchQuery: 'active').listen((_) {});
    await pumpEventQueue();

    for (var index = 0; index < rustFeedLiveLimit; index++) {
      await source.loadRemoteFeed(searchQuery: 'other-$index');
    }

    expect(port.closedFeedIds.map((feed) => feed.value), ['2']);
    await watch.cancel();
    expect(port.closedFeedIds.map((feed) => feed.value), contains('1'));
  });
}
