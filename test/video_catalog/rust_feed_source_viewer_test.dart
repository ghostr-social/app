import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  // The Rust main feed is viewer-scoped (api/feed_types.rs): without the
  // signed-in key the engine cannot route the query to the viewer's
  // follows, so the source must name the viewer on every main feed.
  test('names the signed-in viewer on the main feed it opens', () async {
    final port = FakeRustFeedPort(updates: [
      rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
    ]);
    final source = RustFeedRemoteSource(
      port: port,
      viewer: () => NostrPublicKeyHex.parse(testViewerPublicKey),
    );

    final posts = await source.loadRemoteFeed();

    expect(posts, hasLength(1));
    expect(port.openedSpecs.single.kind, 'main');
    expect(port.openedSpecs.single.viewerPubkey, testViewerPublicKey);
  });
}
