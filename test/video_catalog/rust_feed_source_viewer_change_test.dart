import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // The app graph is built once, before any session is restored, and
  // outlives every sign-out: a viewer captured at construction would
  // keep serving the previous account's follows.
  test('opens the next main feed for the account signed in now', () async {
    var viewer = NostrPublicKeyHex.parse(testViewerPublicKey);
    final port = FakeRustFeedPort(updates: [
      rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
    ]);
    final source = RustFeedRemoteSource(port: port, viewer: () => viewer);

    await source.loadRemoteFeed();
    viewer = NostrPublicKeyHex.parse(testCreatorPublicKey);
    await source.loadRemoteFeed();

    expect(
      port.openedSpecs.map((spec) => spec.viewerPubkey),
      [testViewerPublicKey, testCreatorPublicKey],
    );
  });
}
