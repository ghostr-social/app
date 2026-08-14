import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/account_scoped_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/repost_hydrated_video_feed_repository.dart';

import '../support/fake_video_catalog_repository.dart';

void main() {
  test('account guard preserves existing repost hydration', () {
    final base = FakeVideoCatalogRepository(forYouFeed: []);
    final hydrated = RepostHydratedVideoFeedRepository(base, base);
    final guarded = AccountScopedVideoFeedRepository(hydrated, () => null);

    final result = ensureRepostHydratedVideoFeed(guarded, base);

    expect(result, same(guarded));
  });
}
