import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/profile_video_grid.dart';

import '../support/fakes.dart';
import '../support/recording_video_playback_port.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('a tapped profile video plays as that creator\'s feed',
      (tester) async {
    final creator = sampleCreator();
    final first = samplePost(id: 'clip-1', caption: 'First clip', creator: creator);
    final second =
        samplePost(id: 'clip-2', caption: 'Second clip', creator: creator);
    final playback = RecordingVideoPlaybackPort();
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [first, second],
        feed: FakeFeedScenario(
          profiles: {
            creator.id:
                sampleProfileDetails(profile: creator, posts: [first, second]),
          },
        ),
      ),
      device: FakeDeviceDependencies(playback: playback),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Open profile').first);
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.byKey(ProfileVideoGrid.tileKey(second.id)));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(ProfileVideoGrid.tileKey(second.id)));
    await tester.pumpAndSettle();

    expect(find.byType(ProfileFeedScreen), findsOneWidget);
    expect(find.widgetWithText(AppBar, creator.displayName), findsOneWidget);
    expect(playback.activity[second.media.debugLabel]!.last, isTrue);

    await tester.pageBack();
    await tester.pumpAndSettle();
    expect(find.byType(ProfileScreen), findsOneWidget);
  });
}
