import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('uses an app-safe notice for an unexpected like error', () async {
    final post = samplePost();
    final repository = _UnexpectedLikeRepository(post);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    await cubit.toggleLike(post);

    expect(
      (cubit.state as FeedLoaded).notice,
      'Could not update this like.',
    );
  });
}

class _UnexpectedLikeRepository extends FakeVideoCatalogRepository {
  _UnexpectedLikeRepository(VideoPost post) : super(forYouFeed: [post]);

  @override
  Future<VideoPost> toggleLike(VideoPost post) {
    throw StateError('signer unavailable');
  }
}
