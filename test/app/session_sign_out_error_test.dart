import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('keeps the profile open when secure sign out fails',
      (tester) async {
    final session = sampleSession();
    final dependencies = buildFakeDependencies(
      sessionRepository: FakeSessionRepository(
        storedSession: session,
        signOutFailure: const AppFailure('Could not sign out securely.'),
      ),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [samplePost()],
        feed: FakeFeedScenario(
          profiles: {
            session.profile.id: ProfileDetails(
              profile: session.profile,
              posts: const [],
              statistics: ProfileStatistics(totalLikes: 0, followingCount: 0),
              relationship: ProfileRelationship(
                isFollowing: false,
                isBlocked: false,
                isCurrentUser: true,
              ),
            ),
          },
        ),
      ),
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Profile').last);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Sign out'));
    await tester.pump();

    expect(find.text('Could not sign out securely.'), findsOneWidget);
    expect(find.text('Sign out'), findsOneWidget);
  });
}
