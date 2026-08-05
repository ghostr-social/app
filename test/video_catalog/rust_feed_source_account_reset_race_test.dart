import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/blocking_close_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('an account switch rejects an older load before it opens late',
      () async {
    NostrPublicKeyHex? viewer;
    final port = BlockingCloseRustFeedPort(
      rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
    );
    addTearDown(port.dispose);
    final source = RustFeedRemoteSource(port: port, viewer: () => viewer);
    await source.loadRemoteFeed();

    viewer = NostrPublicKeyHex.parse(testViewerPublicKey);
    final stale = source.loadRemoteFeed();
    await port.closeStarted;
    viewer = NostrPublicKeyHex.parse(testCreatorPublicKey);

    expect(await source.loadRemoteFeed(), hasLength(1));
    port.releaseClose();
    await expectLater(stale, throwsA(isA<AppFailure>()));
    expect(port.openedViewers, [null, testCreatorPublicKey]);
  });
}
