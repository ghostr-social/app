import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/follow_profile_workflow.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'does not offer follows when membership cannot be established',
    () async {
      final repository = _UnreadableFollowRepository(
        forYouFeed: [samplePost()],
      );
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

      final loaded = cubit.state as FeedLoaded;
      expect(
        loaded.canFollow(repository.forYouFeed.single.creator.id),
        isFalse,
      );
    },
  );
}

final class _UnreadableFollowRepository extends FakeVideoCatalogRepository {
  _UnreadableFollowRepository({required super.forYouFeed});

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() {
    throw const AppFailure('Could not load follows.');
  }
}
