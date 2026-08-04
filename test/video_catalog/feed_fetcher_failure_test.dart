import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_fetcher.dart';

void main() {
  test('every fetch failure comes back as a reason the viewer can read',
      () async {
    final fetcher = FeedFetcher(_FailingFeedRepository(
      const AppFailure('No relay reachable.'),
    ));

    final failed = await fetcher.unwatched(FeedKind.forYou);

    expect(failed, isA<FeedUnavailable>());
    expect((failed as FeedUnavailable).describe(), 'No relay reachable.');
  });

  test('an unexpected error never reaches the viewer raw', () async {
    final fetcher = FeedFetcher(_FailingFeedRepository(StateError('boom')));

    final failed = await fetcher.older(FeedKind.forYou, DateTime(2026, 3, 12));

    expect(
      (failed as FeedUnavailable).describe(),
      'Could not load the Nostr video feed.',
    );
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
