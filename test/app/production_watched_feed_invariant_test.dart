import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/fake_nostr_social_port.dart';
import '../support/fake_nostr_video_publisher_port.dart';
import '../support/fake_remote_video_source.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';
import '../support/test_video_delivery.dart';
import '../support/test_watch_history_database.dart';

void main() {
  test(
    'production For You excludes watched rows on load and refresh',
    () async {
      SharedPreferences.setMockInitialValues({
        'ghostr.settings.hideWatchedVideos': false,
      });
      final watched = samplePost(id: 'watched');
      final fresh = samplePost(id: 'fresh');
      final source = FakeRemoteVideoSource([watched, fresh]);
      final nostr = ProductionNostrServices(
        ProductionNostrAdapters(FakeNostrSessionPort(), FakeNostrSocialPort()),
        FakeNostrEventClient(publicKeyHex: testViewerPublicKey),
        FakeNostrVideoPublisherPort(),
      );
      final dependencies = await buildProductionDependencies(
        ProductionDependenciesEnvironment(
          preferencesLoader: SharedPreferences.getInstance,
          nostrServicesBuilder: (_) => nostr,
          videoDeliveryBuilder: (_, __) async =>
              testVideoDelivery(remoteSource: source),
          watchHistoryDatabaseLoader: openTestWatchHistoryDatabase,
        ),
      );
      await dependencies.watchHistoryRepository.record(
        WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 15)),
      );
      final feed = dependencies.videoCatalogServices.feed;

      expect(
        (await feed.loadFeed(FeedKind.forYou)).map((post) => post.id.value),
        ['fresh'],
      );
      final refresh = await (feed as VideoFeedRefreshRepository).loadRefresh(
        FeedKind.forYou,
      );
      expect(refresh.eligiblePosts.map((post) => post.id.value), ['fresh']);
    },
  );
}
