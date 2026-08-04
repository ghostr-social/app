import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('serves an empty list when the feed stream ends without a page',
      () async {
    final port = FakeRustFeedPort(updates: [rustFeedBaseline()]);
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed(searchQuery: 'ghost');

    expect(posts, isEmpty);
    expect(port.closedFeedIds, [port.feedId]);
  });
}
