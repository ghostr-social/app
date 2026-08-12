import 'package:flutter_test/flutter_test.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/feed_screen_harness.dart';
import '../support/recording_video_playback_port.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('comments pause feed playback until the sheet closes', (
    tester,
  ) async {
    final post = samplePost();
    final playback = RecordingVideoPlaybackPort();
    final repository = FakeVideoCatalogRepository(forYouFeed: [post]);
    await tester.pumpWidget(
      feedScreenHarness(
        repository,
        options: FeedScreenHarnessOptions(playbackPort: playback),
      ),
    );
    await tester.pumpAndSettle();
    expect(playback.activity[post.media.debugLabel]!.last, isTrue);

    await tester.tap(find.byTooltip('Open comments'));
    await tester.pumpAndSettle();
    expect(playback.activity[post.media.debugLabel]!.last, isFalse);

    await tester.binding.handlePopRoute();
    await tester.pumpAndSettle();
    expect(playback.activity[post.media.debugLabel]!.last, isTrue);
  });
}
