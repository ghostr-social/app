import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/following_feed_scope_fixture.dart';

void main() {
  test('zero-follow Following load settles without opening a feed', () async {
    final port = FakeRustFeedPort();
    final source = RustFeedRemoteSource(port: port);

    final posts = await source.loadFollowingRemoteFeed(
      testFollowingFeedScope(creators: const {}),
    );

    expect(posts, isEmpty);
    expect(port.openedSpecs, isEmpty);
    expect(port.capturedAccounts, isEmpty);
  });
}
