import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('keeps the native feed open until its scheduled retry lands', () async {
    final post = rustFeedPost();
    final port = FakeRustFeedPort(
      updates: [
        rustFeedBaseline(),
        rustFeedUpdate(revision: 1, stage: FfiFeedStage.failed),
        rustFeedUpdate(revision: 2, stage: FfiFeedStage.loading),
        rustFeedUpdate(revision: 3, posts: [post]),
      ],
    )..closeStreamAfterUpdates = false;
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadRemoteFeed();

    expect(posts.single.id.value, post.eventId);
    expect(port.closedFeedIds, isEmpty);
  });
}
