import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a failed block keeps the feed intact and shows the failure', () async {
    final repository = _FailingBlockRepository([samplePost()]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
      optional: FeedOptionalDependencies(social: repository),
    ));
    addTearDown(cubit.close);
    await cubit.load();

    await cubit.blockCreator(repository.forYouFeed.single);

    final state = cubit.state as FeedLoaded;
    expect(state.posts, hasLength(1));
    expect(state.notice, 'Could not update the Nostr mute list.');
  });
}

class _FailingBlockRepository extends FakeVideoCatalogRepository {
  _FailingBlockRepository(List<VideoPost> posts) : super(forYouFeed: posts);

  @override
  Future<bool> toggleBlock(ProfileId profileId) {
    throw const AppFailure('Could not update the Nostr mute list.');
  }
}
