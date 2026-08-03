import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('reverts the optimistic like when the relay mutation fails', () async {
    final post = samplePost();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [post],
      writes: const FakeWriteScenario(
        likeFailure: AppFailure('Relays are unreachable.'),
      ),
    );
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    await cubit.toggleLike(post);

    final state = cubit.state as FeedLoaded;
    expect(state.posts.first.viewerHasLiked, isFalse);
    expect(state.posts.first.likeCount, post.likeCount);
    expect(state.notice, 'Relays are unreachable.');
  });
}
