import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('a native load-more watcher failure retires the feed', () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
    ]);
    final source = RustFeedRemoteSource(port: port);

    await expectLater(
      source.loadMoreRemoteFeed(searchQuery: 'ghost'),
      throwsA(isA<AppFailure>()),
    );

    expect(port.loadMoreCursors, [null]);
    expect(port.closedFeedIds, [port.feedId]);
  });
}
