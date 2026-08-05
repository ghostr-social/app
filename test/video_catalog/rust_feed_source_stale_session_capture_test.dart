import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';

import '../support/live_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('an older native session cannot adopt after the current one', () async {
    var viewer = NostrPublicKeyHex.parse(testViewerPublicKey);
    final captureStarted = Completer<void>();
    final staleCapture = Completer<RustFeedAccountSession>();
    final currentAccount = NostrPublicKeyHex.parse(testCreatorPublicKey);
    final currentSession = RustFeedAccountSession(
      account: currentAccount,
      generation: RustNostrSessionGeneration.fromBridge(BigInt.two),
    );
    final port = LiveRustFeedPort(
      firstPage: [
        rustFeedUpdate(revision: 1, posts: [rustFeedPost()]),
      ],
      sessionCapture: (account) {
        if (account == currentAccount) return Future.value(currentSession);
        captureStarted.complete();
        return staleCapture.future;
      },
    );
    final source = RustFeedRemoteSource(port: port, viewer: () => viewer);

    final stale = source.loadRemoteFeed();
    await captureStarted.future;
    viewer = currentAccount;
    expect(await source.loadRemoteFeed(), hasLength(1));
    staleCapture.complete(
      RustFeedAccountSession(
        account: NostrPublicKeyHex.parse(testViewerPublicKey),
        generation: RustNostrSessionGeneration.fromBridge(BigInt.one),
      ),
    );

    await expectLater(stale, throwsA(isA<AppFailure>()));
    expect(port.openedSpecs.single.viewerPubkey, testCreatorPublicKey);
    expect(port.closedFeedIds, isEmpty);
  });
}
