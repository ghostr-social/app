import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_backfill.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_fetcher.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_loads.dart';

import '../support/sample_data.dart';

void main() {
  test('a page that lands after a newer load is thrown away', () async {
    final loads = FeedLoads();
    final repository = _PendingOlderFeedRepository();
    final backfill = FeedBackfill(FeedFetcher(repository), loads);
    backfill.restartFrom([samplePost(id: 'post-0')]);
    loads.take();

    final dig = backfill.dig(FeedKind.forYou);
    loads.take();
    repository.pending.complete(VideoFeedPage(
      posts: [samplePost(id: 'older-0')],
      nextOlderThan: DateTime(2026, 3, 11),
    ));

    expect(await dig, isA<FeedDigSkipped>());
  });
}

class _PendingOlderFeedRepository implements VideoFeedRepository {
  final pending = Completer<VideoFeedPage>();

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    return const <VideoPost>[];
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) {
    return pending.future;
  }
}
