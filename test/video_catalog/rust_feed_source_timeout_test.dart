import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // A feed that never produces a page (blank-search parity in the Rust
  // store) must still answer instead of hanging the caller.
  test('returns the baseline snapshot when no revision lands in time', () {
    fakeAsync((async) {
      final port = FakeRustFeedPort(updates: [rustFeedUpdate(revision: 0)])
        ..closeStreamAfterUpdates = false;
      final source = RustFeedRemoteSource(port: port);
      List<VideoPost>? result;

      source
          .loadRemoteFeed(searchQuery: 'ghost')
          .then((posts) => result = posts);
      async.elapse(const Duration(seconds: 9));

      expect(result, isEmpty);
      expect(port.closedFeedIds, [port.feedId]);
    });
  });
}
