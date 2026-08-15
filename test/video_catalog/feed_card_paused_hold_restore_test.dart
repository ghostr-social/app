import 'package:flutter/gestures.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/feed_screen_harness.dart';
import '../support/recording_video_playback_port.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('left-edge acceleration restores the prior paused mode', (
    tester,
  ) async {
    final post = samplePost();
    final playback = RecordingVideoPlaybackPort();
    await tester.pumpWidget(
      feedScreenHarness(
        FakeVideoCatalogRepository(forYouFeed: [post]),
        options: FeedScreenHarnessOptions(playbackPort: playback),
      ),
    );
    await tester.pumpAndSettle();
    final card = tester.getRect(find.byType(FeedCard));
    await tester.tapAt(card.center);
    await tester.pump();

    final hold = await tester.startGesture(
      card.centerLeft + const Offset(8, 0),
    );
    await tester.pump(kLongPressTimeout + const Duration(milliseconds: 50));
    expect(
      playback.modes[post.media.debugLabel]!.last,
      VideoPlaybackMode.accelerated,
    );
    await hold.up();
    await tester.pump();

    expect(
      playback.modes[post.media.debugLabel]!.last,
      VideoPlaybackMode.paused,
    );
    expect(find.bySemanticsLabel('Play video'), findsOneWidget);
  });
}
