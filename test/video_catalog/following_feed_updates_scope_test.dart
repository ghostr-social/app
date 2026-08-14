import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';

import '../support/discovery_search_fakes.dart';
import '../support/fake_remote_video_source.dart';
import '../support/following_feed_scope_fixture.dart';
import '../support/sample_data.dart';

void main() {
  test('following feed revisions stay scoped to followed creators', () async {
    final creator = sampleCreator(id: 'npub1followed');
    final remote = FakeRemoteVideoSource([samplePost(creator: creator)]);
    final social = FakeSocialGraph()..followed.add(creator.id);
    final updates = RemoteVideoFeedUpdates(
      remote: remote,
      followingScopes: testFollowingFeedScopes(social),
    );

    final update = await updates.watchFeed(FeedKind.following).first;

    expect(remote.requestedWatchCreatorIds, {creator.id});
    expect(update.phase, VideoFeedUpdatePhase.settled);
    expect(update.hasPosts, isTrue);
  });
}
