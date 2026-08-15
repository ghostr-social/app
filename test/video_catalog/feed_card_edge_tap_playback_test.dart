import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/feed_screen_harness.dart';
import '../support/recording_video_playback_port.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('tapping the video edge leaves playback running', (tester) async {
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

    await tester.tapAt(card.centerLeft + const Offset(8, 0));
    await tester.pump();

    expect(
      playback.modes[post.media.debugLabel]!.last,
      VideoPlaybackMode.normal,
    );
    expect(find.bySemanticsLabel('Pause video'), findsOneWidget);
  });
}
