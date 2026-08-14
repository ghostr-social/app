import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('zero-follow Following updates settle without reconnecting', () async {
    final port = FakeRustFeedPort();
    final source = RustFeedRemoteSource(port: port);
    final snapshots = <RemoteVideoSnapshot>[];
    var completed = false;
    final subscription = source
        .watchFollowingRemoteFeed(
          FollowingFeedScope(
            viewer: NostrPublicKeyHex.parse(testViewerPublicKey),
            creators: const {},
          ),
        )
        .listen(snapshots.add, onDone: () => completed = true);
    addTearDown(subscription.cancel);

    await pumpEventQueue();

    expect(snapshots, hasLength(1));
    expect(snapshots.single.phase, RemoteVideoPhase.settled);
    expect(snapshots.single.posts, isEmpty);
    expect(completed, isFalse);
    expect(port.openedSpecs, isEmpty);
  });
}
