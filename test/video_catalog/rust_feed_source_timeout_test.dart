import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_page_reader.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // A feed whose page never settles must still answer instead of
  // hanging the caller — and it must not answer early, while the
  // pipeline's slowest query could still deliver the page.
  test('answers once the deadline passes without a settled page', () {
    fakeAsync((async) {
      final port = FakeRustFeedPort(updates: [rustFeedBaseline()])
        ..closeStreamAfterUpdates = false;
      final source = RustFeedRemoteSource(port: port);
      List<VideoPost>? result;

      source
          .loadRemoteFeed(searchQuery: 'ghost')
          .then((posts) => result = posts);
      async.elapse(rustDiscoveryQueryTimeout);
      expect(result, isNull, reason: 'the plan may still be running');

      async.elapse(rustFeedPageDeadline);
      expect(result, isEmpty);
      expect(
        port.closedFeedIds,
        isEmpty,
        reason: 'the page is still coming; the next pull collects it',
      );
    });
  });
}
