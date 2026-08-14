import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_feed_updates.dart';

import '../support/discovery_search_fakes.dart';
import '../support/fake_remote_video_source.dart';
import '../support/following_feed_scope_fixture.dart';

void main() {
  test('a creator-scoped feed replaces an active unscoped feed', () async {
    final social = FakeSocialGraph()
      ..followed.add(ProfileId.parse('npub1creator'));
    final updates = RemoteVideoFeedUpdates(
      remote: FakeRemoteVideoSource([]),
      followingScopes: testFollowingFeedScopes(social),
    );
    await updates.watchFeed(FeedKind.forYou).first;

    final shouldRebind = await updates.shouldRebind(FeedKind.following);

    expect(shouldRebind, isTrue);
  });
}
