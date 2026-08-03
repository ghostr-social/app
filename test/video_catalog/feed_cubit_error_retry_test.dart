import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  blocTest<FeedCubit, FeedState>(
    'emits a safe feed error and reloads the current feed on retry',
    build: () {
      final repository = _RetryingFeedRepository();
      return FeedCubit(FeedDependencies(
        feed: repository,
        engagement: repository,
      ));
    },
    act: (cubit) async {
      await cubit.load(FeedKind.following);
      await cubit.retry();
    },
    expect: () => [
      isA<FeedLoading>(),
      isA<FeedFailure>()
          .having((state) => state.message, 'message', 'Feed failed.'),
      isA<FeedLoading>(),
      isA<FeedLoaded>(),
    ],
  );
}

class _RetryingFeedRepository extends FakeVideoCatalogRepository {
  _RetryingFeedRepository() : super(forYouFeed: [samplePost()]);

  int count = 0;

  @override
  Future<List<VideoPost>> loadFeed(FeedKind kind,
      {bool excludeWatched = false}) async {
    count += 1;
    if (count == 1) throw const AppFailure('Feed failed.');
    return [samplePost()];
  }
}
