import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';

void main() {
  test('uses an app-safe message for an unexpected feed load error', () async {
    final repository = _UnexpectedFeedRepository();
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);

    await cubit.load();

    expect(
      (cubit.state as FeedFailure).message,
      'Could not load the Nostr video feed.',
    );
  });
}

class _UnexpectedFeedRepository extends FakeVideoCatalogRepository {
  _UnexpectedFeedRepository() : super(forYouFeed: []);

  @override
  Future<List<VideoPost>> loadFeed(FeedKind kind,
      {bool excludeWatched = false}) {
    throw StateError('relay unavailable');
  }
}
