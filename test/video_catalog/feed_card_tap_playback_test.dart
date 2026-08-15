import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/feed_screen_harness.dart';
import '../support/recording_video_playback_port.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('tapping the video center pauses and resumes playback', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    final post = samplePost();
    final playback = RecordingVideoPlaybackPort();
    await tester.pumpWidget(
      feedScreenHarness(
        FakeVideoCatalogRepository(forYouFeed: [post]),
        options: FeedScreenHarnessOptions(playbackPort: playback),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.bySemanticsLabel('Pause video'), findsOneWidget);
    await tester.tapAt(tester.getCenter(find.byType(FeedCard)));
    await tester.pump();
    expect(
      playback.modes[post.media.debugLabel]!.last,
      VideoPlaybackMode.paused,
    );
    expect(find.bySemanticsLabel('Play video'), findsOneWidget);

    await tester.tapAt(tester.getCenter(find.byType(FeedCard)));
    await tester.pump();
    expect(
      playback.modes[post.media.debugLabel]!.last,
      VideoPlaybackMode.normal,
    );
    expect(find.bySemanticsLabel('Pause video'), findsOneWidget);
    semantics.dispose();
  });
}
