import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

import '../support/fakes.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('renders the current-user profile and signs out', (tester) async {
    var signedOut = false;
    final session = sampleSession();
    final catalog = FakeVideoCatalogRepository(
      forYouFeed: [],
      feed: FakeFeedScenario(profiles: {
        session.profile.id: ProfileDetails(
          profile: session.profile,
          posts: [
            samplePost(caption: 'My profile clip', creator: session.profile),
          ],
          statistics: ProfileStatistics(totalLikes: 7, followingCount: 2),
          relationship: ProfileRelationship(
            isFollowing: false,
            isBlocked: false,
            isCurrentUser: true,
          ),
        ),
      }),
    );
    await tester.pumpWidget(profileScreenHarness(
      profile: catalog,
      viewer: session.profile,
      profileId: session.profile.id,
      onSignedOut: () => signedOut = true,
    ));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Sign out'));

    expect(find.text('My profile clip'), findsOneWidget);
    expect(signedOut, isTrue);
  });
}
