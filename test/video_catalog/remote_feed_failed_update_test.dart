import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';

import '../support/discovery_search_fakes.dart';

void main() {
  test('a failed global feed update keeps its unscoped subscription', () async {
    final updates = RemoteVideoFeedUpdates(
      remote: _FailedRemoteUpdates(),
      social: FakeSocialGraph(),
    );

    final update = await updates.watchFeed(FeedKind.forYou).first;

    expect(update.phase, VideoFeedUpdatePhase.failed);
    expect(update.hasPosts, isFalse);
    expect(await updates.shouldRebind(FeedKind.forYou), isFalse);
  });
}

final class _FailedRemoteUpdates implements RemoteVideoUpdates {
  @override
  Stream<RemoteVideoSnapshot> watchRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) {
    return Stream.value(
      RemoteVideoSnapshot(
        revision: BigInt.one,
        phase: RemoteVideoPhase.failed,
        posts: const [],
      ),
    );
  }
}
