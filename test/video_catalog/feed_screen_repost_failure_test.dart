import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/repost_samples.dart';

void main() {
  testWidgets('shows repost failure and restores the feed action', (
    tester,
  ) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [repostablePost()],
    );
    await tester.pumpWidget(
      feedScreenHarness(
        repository,
        options: const FeedScreenHarnessOptions(reposts: _FailingReposts()),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Repost video'));
    await tester.pumpAndSettle();

    expect(find.text('Relays are unreachable.'), findsOneWidget);
    expect(find.byTooltip('Repost video'), findsOneWidget);
  });
}

final class _FailingReposts implements VideoRepostRepository {
  const _FailingReposts();

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) async => posts;

  @override
  Future<VideoPost> toggleRepost(VideoPost post) {
    throw const AppFailure('Relays are unreachable.');
  }
}
