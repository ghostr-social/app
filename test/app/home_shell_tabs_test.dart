import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('switches across the non-home shell tabs', (tester) async {
    final session = sampleSession();
    final dependencies = buildFakeDependencies(
      session: session,
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [samplePost()],
        feed: FakeFeedScenario(profiles: {
          session.profile.id: ProfileDetails(
            profile: session.profile,
            posts: [samplePost(caption: 'Own clip', creator: session.profile)],
            statistics: ProfileStatistics(totalLikes: 4, followingCount: 1),
            relationship: ProfileRelationship(
              isFollowing: false,
              isBlocked: false,
              isCurrentUser: true,
            ),
          ),
        }),
      ),
      device: FakeDeviceDependencies(
        activity: FakeActivityRepository(items: [sampleActivity()]),
        mediaPicker: FakeMediaPickerPort(galleryMedia: sampleMedia()),
      ),
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();
    expect(find.text('Search creators and videos'), findsOneWidget);
    await tester.tap(find.text('Create'));
    await tester.pumpAndSettle();
    expect(find.text('Choose from library'), findsOneWidget);
    await tester.tap(find.text('Activity'));
    await tester.pumpAndSettle();
    expect(find.text('Published a video'), findsOneWidget);
    await tester.tap(find.text('Profile'));
    await tester.pumpAndSettle();
    expect(find.text('Sign out'), findsOneWidget);
  });
}
