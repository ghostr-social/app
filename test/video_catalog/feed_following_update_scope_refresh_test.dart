import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/discovery_search_fakes.dart';
import '../support/fakes.dart';
import '../support/following_feed_scope_fixture.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';

void main() {
  test('Following refresh rebinds on creator scope changes', () async {
    final first = sampleCreator(id: 'npub1first');
    final second = sampleCreator(id: 'npub1second');
    final social = FakeSocialGraph()..followed.add(first.id);
    final remote = _WarmRemoteUpdates();
    final updates = RemoteVideoFeedUpdates(
      remote: remote,
      followingScopes: testFollowingFeedScopes(social),
    );
    final feed = ScriptedFeedRepository(
      loads: [
        [samplePost(id: 'initial', creator: first)],
        [samplePost(id: 'initial', creator: first)],
        [samplePost(id: 'initial', creator: first)],
      ],
    );
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: []),
        optional: FeedOptionalDependencies(
          delivery: FeedDeliveryDependencies(updates: updates),
        ),
      ),
    );
    addTearDown(cubit.close);
    addTearDown(remote.close);
    await cubit.load(FeedKind.following);
    Future<void> refresh() async {
      final pending = cubit.refresh();
      await pumpEventQueue();
      remote.releaseCancellation();
      await pending;
      await pumpEventQueue();
    }

    await refresh();
    social.followed.add(second.id);
    await refresh();
    expect(remote.scopes, [
      {first.id},
      {first.id, second.id},
    ]);
    expect(remote.cancellations, 1);
  });
}

final class _WarmRemoteUpdates implements RemoteVideoUpdates {
  _WarmRemoteUpdates() {
    _controller = StreamController.broadcast(
      onCancel: () => cancellations += 1,
    );
  }

  late final StreamController<RemoteVideoSnapshot> _controller;
  final scopes = <Set<ProfileId>?>[];
  int cancellations = 0;
  int revision = 0;

  @override
  Stream<RemoteVideoSnapshot> watchRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) {
    scopes.add(creatorIds == null ? null : {...creatorIds});
    return _controller.stream;
  }

  void releaseCancellation() {
    revision += 1;
    _controller.add(
      RemoteVideoSnapshot(
        revision: BigInt.from(revision),
        phase: RemoteVideoPhase.settled,
        posts: const [],
      ),
    );
  }

  Future<void> close() => _controller.close();
}
