import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/profile_video_grid.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fakes.dart';
import '../support/recording_video_playback_port.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('moderation sheet keeps a routed video visible and active', (
    tester,
  ) async {
    final creator = sampleCreator();
    final entry = samplePost(id: 'entry', creator: creator);
    final routed = samplePost(id: 'routed', creator: creator);
    final playback = RecordingVideoPlaybackPort();
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [entry],
        feed: FakeFeedScenario(
          profiles: {
            creator.id: sampleProfileDetails(
              profile: creator,
              posts: [entry, routed],
            ),
          },
        ),
      ),
      device: FakeDeviceDependencies(playback: playback),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Open profile'));
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.byKey(ProfileVideoGrid.tileKey(routed.id)));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(ProfileVideoGrid.tileKey(routed.id)));
    await tester.pumpAndSettle();
    final disposalsBeforeSheet = playback.surfaceDisposals;
    await tester.longPress(find.byType(FeedCard));
    await tester.pumpAndSettle();

    expect(find.text('Cancel'), findsOneWidget);
    expect(playback.activity[routed.media.debugLabel], everyElement(isTrue));
    expect(playback.surfaceDisposals, disposalsBeforeSheet);
    expect(
      playback.modes[routed.media.debugLabel]!.last,
      VideoPlaybackMode.normal,
    );
    final barrier = tester.widget<ModalBarrier>(find.byType(ModalBarrier).last);
    expect(barrier.color!.a, lessThanOrEqualTo(0.25));
  });
}
