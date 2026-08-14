import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/account_scoped_video_feed_repository.dart';

import '../support/fake_video_catalog_repository.dart';

void main() {
  test('reports false when the wrapped feed has no hydration status', () {
    final feed = FakeVideoCatalogRepository(forYouFeed: []);
    final guarded = AccountScopedVideoFeedRepository(feed, () => null);

    expect(guarded.isRepostHydrated, isFalse);
  });
}
