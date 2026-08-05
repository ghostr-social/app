import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_fetcher.dart';

void main() {
  test('every fetch failure retains its cause for the presenter', () async {
    const cause = AppFailure('No relay reachable.');
    final fetcher = FeedFetcher(_FailingFeedRepository(cause));

    final failed = await fetcher.unwatched(FeedKind.forYou);

    expect(failed, isA<FeedUnavailable>());
    expect((failed as FeedUnavailable).failure.cause, same(cause));
  });

  test('an unexpected error also retains its diagnostic context', () async {
    final cause = StateError('boom');
    final fetcher = FeedFetcher(_FailingFeedRepository(cause));

    final failed = await fetcher.older(FeedKind.forYou, DateTime(2026, 3, 12));

    final failure = (failed as FeedUnavailable).failure;
    expect(failure.cause, same(cause));
    expect(failure.stackTrace, isNot(StackTrace.empty));
  });
}

class _FailingFeedRepository implements VideoFeedRepository {
  _FailingFeedRepository(this.error);

  final Object error;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    throw error;
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    throw error;
  }
}
