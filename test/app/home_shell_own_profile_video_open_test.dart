import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/profile_video_grid.dart';

import '../support/fakes.dart';
import '../support/recording_video_playback_port.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('the own profile tab plays a tapped video as a feed',
      (tester) async {
    final session = sampleSession();
    final me = session.profile;
    final clip = samplePost(id: 'mine-1', caption: 'My clip', creator: me);
    final playback = RecordingVideoPlaybackPort();
    final dependencies = buildFakeDependencies(
      session: session,
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [clip],
        feed: FakeFeedScenario(
          profiles: {
            me.id: sampleProfileDetails(
              profile: me,
              posts: [clip],
              relationship: ProfileRelationship(
                isFollowing: false,
                isBlocked: false,
                isCurrentUser: true,
              ),
            ),
          },
        ),
      ),
      device: FakeDeviceDependencies(playback: playback),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Profile'));
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.byKey(ProfileVideoGrid.tileKey(clip.id)));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(ProfileVideoGrid.tileKey(clip.id)));
    await tester.pumpAndSettle();

    expect(find.byType(ProfileFeedScreen), findsOneWidget);
    expect(playback.activity[clip.media.debugLabel]!.last, isTrue);
  });
}
