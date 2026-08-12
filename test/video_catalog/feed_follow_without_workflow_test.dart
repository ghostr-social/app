import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('does not offer follows without a follow workflow', () async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
    final cubit = FeedCubit(
      FeedDependencies(
        viewerId: sampleSession().profile.id,
        feed: repository,
        engagement: repository,
        optional: FeedOptionalDependencies(social: repository),
      ),
    );
    addTearDown(cubit.close);

    await cubit.load();

    final creator = repository.forYouFeed.single.creator;
    expect((cubit.state as FeedLoaded).canFollow(creator.id), isFalse);
  });
}
