import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/account_scoped_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test('returns a feed completed for the same account', () async {
    final post = samplePost();
    final feed = FakeVideoCatalogRepository(forYouFeed: [post]);
    final viewer = NostrPublicKeyHex.parse(testViewerPublicKey);
    final guarded = AccountScopedVideoFeedRepository(feed, () => viewer);

    final posts = await guarded.loadFeed(FeedKind.forYou);

    expect(posts, [post]);
  });
}
