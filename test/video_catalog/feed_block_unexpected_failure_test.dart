import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('uses an app-safe notice for an unexpected block error', () async {
    final repository = _BrokenBlockRepository([samplePost()]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
      optional: FeedOptionalDependencies(social: repository),
    ));
    addTearDown(cubit.close);
    await cubit.load();

    await cubit.blockCreator(repository.forYouFeed.single);

    expect(
      (cubit.state as FeedLoaded).notice,
      'Could not block this creator.',
    );
  });
}

class _BrokenBlockRepository extends FakeVideoCatalogRepository {
  _BrokenBlockRepository(List<VideoPost> posts) : super(forYouFeed: posts);

  @override
  Future<bool> toggleBlock(ProfileId profileId) {
    throw StateError('mute list serializer crashed');
  }
}
