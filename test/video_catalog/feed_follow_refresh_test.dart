import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/follow_profile_workflow.dart';
import '../support/sample_data.dart';

void main() {
  test('refresh reconciles follows changed from another screen', () async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
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
  });
}
