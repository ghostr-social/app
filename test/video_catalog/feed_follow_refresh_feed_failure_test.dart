import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/follow_profile_workflow.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'a failed feed refresh keeps newly reconciled follow membership',
    () async {
      final repository = _FailingRefreshRepository();
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
      expect((cubit.state as FeedLoaded).canFollow(creator.id), isTrue);

      repository.followedProfiles.add(creator.id);
      await cubit.refresh();

      expect((cubit.state as FeedLoaded).canFollow(creator.id), isFalse);
    },
  );
}

final class _FailingRefreshRepository extends FakeVideoCatalogRepository {
  _FailingRefreshRepository() : super(forYouFeed: [samplePost()]);

  var loads = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    if (loads++ > 0) throw const AppFailure('offline');
    return super.loadFeed(kind, excludeWatched: excludeWatched);
  }
}
