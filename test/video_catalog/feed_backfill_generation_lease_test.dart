import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_backfill.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_fetcher.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_loads.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/sample_data.dart';

void main() {
  test('a stale page cannot release a newer pagination lease', () async {
    final loads = FeedLoads();
    final feed = _GenerationalFeed();
    final backfill = FeedBackfill(FeedFetcher(feed), loads);
    backfill.restartFrom([samplePost(id: 'old-anchor')]);
    loads.take();
    final stale = backfill.dig(FeedKind.forYou);

    loads.take();
    backfill.restartFrom([samplePost(id: 'new-anchor')]);
    final current = backfill.dig(FeedKind.forYou);
    addTearDown(() {
      if (!feed.current.isCompleted) feed.current.complete(_page('current'));
    });
    feed.stale.complete(_page('stale'));
    expect(await stale, isA<FeedDigSkipped>());

    final overlap = await backfill.dig(FeedKind.forYou);

    expect(overlap, isA<FeedDigSkipped>());
    feed.current.complete(_page('current'));
    expect(await current, isA<FeedDigPage>());
  });
}

VideoFeedPage _page(String id) {
  return VideoFeedPage(
    posts: [samplePost(id: id)],
    nextOlderThan: DateTime(2026, 3, 10),
  );
}

final class _GenerationalFeed implements VideoFeedRepository {
  final stale = Completer<VideoFeedPage>();
  final current = Completer<VideoFeedPage>();
  int calls = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    return const [];
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) {
    calls += 1;
    if (calls == 1) return stale.future;
    if (calls == 2) return current.future;
    return Future.value(_page('overlap'));
  }
}
