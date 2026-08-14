import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/following_feed_scope_fixture.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('Following older page stays bound to the scope owner', () async {
    final port = FakeRustFeedPort(
      updates: [rustFeedUpdate(revision: 1)],
      moreAvailable: false,
    );
    final source = RustFeedRemoteSource(port: port);
    final scope = testFollowingFeedScope();

    await source.loadFollowingRemoteFeed(
      scope,
      olderThan: DateTime.utc(2026, 1, 2),
    );

    expect(port.capturedAccounts.single, scope.viewer);
    expect(port.openedSpecs.single.viewerPubkey, scope.viewer.value);
    expect(port.loadMoreCursors, [null]);
  });
}
