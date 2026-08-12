import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

import '../support/pending_profile_loads.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('cached profile stays visible and announces detail refresh', (
    tester,
  ) async {
    final viewer = sampleSession().profile;
    final repository = PendingProfileLoads();
    await tester.pumpWidget(
      profileScreenHarness(
        profile: repository,
        viewer: viewer,
        profileId: viewer.id,
      ),
    );
    await tester.pump();

    expect(find.text(viewer.displayName), findsOneWidget);
    expect(find.text('Refreshing profile'), findsOneWidget);
    expect(find.bySemanticsLabel('Refreshing profile'), findsOneWidget);

    repository.pending.complete(
      ProfileDetails(
        profile: viewer,
        posts: [samplePost(creator: viewer)],
        statistics: ProfileStatistics(totalLikes: 42, followingCount: 3),
        relationship: ProfileRelationship(
          isFollowing: false,
          isBlocked: false,
          isCurrentUser: true,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Refreshing profile'), findsNothing);
    expect(find.text('1 Posts'), findsOneWidget);
  });
}
