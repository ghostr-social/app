import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/feed_screen_harness.dart';
import '../support/recording_video_playback_port.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('an inactive video returns to normal mode when revisited', (
    tester,
  ) async {
    final first = samplePost(id: 'first');
    final second = samplePost(id: 'second');
    final playback = RecordingVideoPlaybackPort();
    await tester.pumpWidget(
      feedScreenHarness(
        FakeVideoCatalogRepository(forYouFeed: [first, second]),
        options: FeedScreenHarnessOptions(playbackPort: playback),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tapAt(tester.getCenter(find.byType(FeedCard).first));
    await tester.pump();
    expect(
      playback.modes[first.media.debugLabel]!.last,
      VideoPlaybackMode.paused,
    );

    final controller = tester
        .widget<PageView>(find.byType(PageView))
        .controller!;
    controller.jumpToPage(1);
    await tester.pumpAndSettle();
    controller.jumpToPage(0);
    await tester.pumpAndSettle();

    expect(
      playback.modes[first.media.debugLabel]!.last,
      VideoPlaybackMode.normal,
    );
    expect(find.bySemanticsLabel('Pause video'), findsOneWidget);
  });
}
