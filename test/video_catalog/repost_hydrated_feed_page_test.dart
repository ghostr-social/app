import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/repost_hydrated_video_feed_repository.dart';

import '../support/repost_hydrating_catalog.dart';
import '../support/repost_samples.dart';

void main() {
  test(
    'hydrates initial and older pages without changing the cursor',
    () async {
      final post = repostablePost();
      final source = RepostHydratingCatalog(forYouFeed: [post]);
      source.olderFeedPages.addAll([
        [post],
        [post],
      ]);
      final hydrated = RepostHydratedVideoFeedRepository(source, source);
      final feed = ensureRepostHydratedVideoFeed(hydrated, source);
      final cursor = DateTime.utc(2026, 1, 1);

      final initial = await feed.loadFeed(FeedKind.forYou);
      final older = await feed.loadOlderFeed(
        FeedKind.forYou,
        olderThan: cursor,
      );

      expect(initial.single.viewerHasReposted, isTrue);
      expect(older.posts.single.viewerHasReposted, isTrue);
      expect(older.nextOlderThan, cursor);
      expect(source.hydratedBatches, 2);
    },
  );
}
