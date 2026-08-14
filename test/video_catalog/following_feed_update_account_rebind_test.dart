import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_feed_updates.dart';

import '../support/discovery_search_fakes.dart';
import '../support/fake_remote_video_source.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test('following update rebinds when only the account changes', () async {
    final social = FakeSocialGraph()..followed.add(sampleCreator().id);
    var viewer = NostrPublicKeyHex.parse(testViewerPublicKey);
    final updates = RemoteVideoFeedUpdates(
      remote: FakeRemoteVideoSource([]),
      followingScopes: FollowingFeedScopeReader(social, () => viewer),
    );
    await updates.watchFeed(FeedKind.following).first;

    viewer = NostrPublicKeyHex.parse(testCreatorPublicKey);

    expect(await updates.shouldRebind(FeedKind.following), isTrue);
  });
}
