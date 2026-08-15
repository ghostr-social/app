import 'package:flutter/gestures.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/feed_screen_harness.dart';
import '../support/recording_video_playback_port.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('holding the extreme left accelerates until release', (
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

    final hold = await tester.startGesture(
      card.centerLeft + const Offset(8, 0),
    );
    await tester.pump(kLongPressTimeout + const Duration(milliseconds: 50));
    expect(
      playback.modes[post.media.debugLabel]!.last,
      VideoPlaybackMode.accelerated,
    );
    expect(find.text('2×'), findsOneWidget);
    expect(find.bySemanticsLabel('Playing at 2x speed'), findsOneWidget);

    await hold.up();
    await tester.pump();
    expect(
      playback.modes[post.media.debugLabel]!.last,
      VideoPlaybackMode.normal,
    );
    expect(find.text('2×'), findsNothing);
  });
}
