import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('updates only the matching post after comments are published', () async {
    final target = samplePost(id: 'target');
    final other = samplePost(id: 'other');
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [target, other],
    );
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    cubit.commentsPublished(target, 2);

    final posts = (cubit.state as FeedLoaded).posts;
    expect(posts.first.commentCount, target.commentCount + 2);
    expect(posts.last, same(other));
  });
}
