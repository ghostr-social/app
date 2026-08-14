import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/account_scoped_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('rejects an older page completed for the previous account', () async {
    final feed = _GatedFeed();
    var viewer = NostrPublicKeyHex.parse(testViewerPublicKey);
    final guarded = AccountScopedVideoFeedRepository(feed, () => viewer);

    final loading = guarded.loadOlderFeed(
      FeedKind.following,
      olderThan: DateTime.utc(2026),
    );
    await feed.started.future;
    viewer = NostrPublicKeyHex.parse(testCreatorPublicKey);
    feed.release.complete();

    await expectLater(loading, throwsA(isA<AppFailure>()));
  });
}

final class _GatedFeed implements VideoFeedRepository {
  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async => const [];

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    started.complete();
    await release.future;
    return VideoFeedPage(posts: const []);
  }
}
