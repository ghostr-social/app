import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/follow_profile_workflow.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'a failed feed follow keeps the action available and reports it',
    () async {
      final repository = _FailingFollowRepository(forYouFeed: [samplePost()]);
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

      await cubit.followCreator(creator);

      final loaded = cubit.state as FeedLoaded;
      expect(loaded.canFollow(creator.id), isTrue);
      expect(loaded.notice, 'Relay rejected the follow.');
    },
  );
}

final class _FailingFollowRepository extends FakeVideoCatalogRepository {
  _FailingFollowRepository({required super.forYouFeed});

  @override
  Future<FollowOutcome> follow(ProfileId profileId) {
    throw const AppFailure('Relay rejected the follow.');
  }
}
