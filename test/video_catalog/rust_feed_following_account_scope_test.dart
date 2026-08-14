import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';

import '../support/fake_rust_feed_port.dart';
import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';

void main() {
  test('Following opens with the account that owns its follow set', () async {
    final port = FakeRustFeedPort(updates: [rustFeedUpdate(revision: 1)]);
    final source = RustFeedRemoteSource(
      port: port,
      viewer: () => NostrPublicKeyHex.parse(testCreatorPublicKey),
    );
    final following = source as FollowingRemoteVideoSource;
    final owner = NostrPublicKeyHex.parse(testViewerPublicKey);
    final scope = FollowingFeedScope(
      viewer: owner,
      creators: {ProfileId.parse(testCreatorNpub)},
    );

    await following.loadFollowingRemoteFeed(scope);

    expect(port.capturedAccounts.single, owner);
    expect(port.openedSpecs.single.viewerPubkey, owner.value);
  });
}
