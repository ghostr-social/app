import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('disables unsupported reposts through the feed screen', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    final repository = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
    final reposts = _RecordingReposts();
    await tester.pumpWidget(
      feedScreenHarness(
        repository,
        options: FeedScreenHarnessOptions(reposts: reposts),
      ),
    );
    await tester.pumpAndSettle();

    const tooltip = 'Reposting unavailable for this video';
    expect(
      tester.getSemantics(find.byTooltip(tooltip)),
      isSemantics(
        tooltip: tooltip,
        hasSelectedState: true,
        isSelected: false,
        isButton: true,
        isEnabled: false,
        hasTapAction: false,
      ),
    );
    await tester.tap(find.byTooltip(tooltip), warnIfMissed: false);
    await tester.pump();
    expect(reposts.toggleCount, 0);
    semantics.dispose();
  });
}

final class _RecordingReposts implements VideoRepostRepository {
  int toggleCount = 0;

  @override
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  }) async => posts;

  @override
  Future<VideoPost> toggleRepost(VideoPost post) async {
    toggleCount += 1;
    return post;
  }
}
