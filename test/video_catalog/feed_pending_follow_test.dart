import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/follow_profile_workflow.dart';
import '../support/sample_data.dart';

void main() {
  test('a pending feed follow cannot be submitted twice', () async {
    final repository = _PendingFollowRepository(forYouFeed: [samplePost()]);
    final cubit = FeedCubit(
      FeedDependencies(
        viewerId: sampleSession().profile.id,
        feed: repository,
        engagement: repository,
        followProfile: testFollowProfileWorkflow(repository),
        optional: FeedOptionalDependencies(social: repository),
      ),
    );
    addTearDown(cubit.close);
    await cubit.load();
    final creator = repository.forYouFeed.single.creator;

    final first = cubit.followCreator(creator);
    final second = cubit.followCreator(creator);

    expect(repository.requests, [creator.id]);
    expect((cubit.state as FeedLoaded).canFollow(creator.id), isFalse);
    repository.pending.complete(FollowOutcome.newlyFollowed);
    await Future.wait([first, second]);
    expect((cubit.state as FeedLoaded).canFollow(creator.id), isFalse);
  });
}

final class _PendingFollowRepository extends FakeVideoCatalogRepository {
  _PendingFollowRepository({required super.forYouFeed});

  final pending = Completer<FollowOutcome>();
  final requests = <ProfileId>[];

  @override
  Future<FollowOutcome> follow(ProfileId profileId) {
    requests.add(profileId);
    return pending.future;
  }
}
