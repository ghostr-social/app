import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/account_scoped_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/repost_hydrated_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test('production decorators preserve filtered refresh eligibility', () async {
    final watched = samplePost(id: 'watched');
    final fresh = samplePost(id: 'fresh');
    final source = FakeVideoCatalogRepository(forYouFeed: [watched, fresh]);
    final aware = WatchAwareVideoFeedRepository(
      feed: source,
      history: FakeWatchHistoryRepository(
        entries: [
          WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 1)),
        ],
      ),
      failureReporter: RecordingFailureReporter(),
    );
    final feed = AccountScopedVideoFeedRepository(
      RepostHydratedVideoFeedRepository(aware, source),
      () => NostrPublicKeyHex.parse(testViewerPublicKey),
    );

    expect(feed, isA<VideoFeedRefreshRepository>());
    final snapshot = await (feed as VideoFeedRefreshRepository).loadRefresh(
      FeedKind.forYou,
    );

    expect(snapshot.allPosts.map((post) => post.id.value), [
      'watched',
      'fresh',
    ]);
    expect(snapshot.eligiblePosts.map((post) => post.id.value), ['fresh']);
  });
}
