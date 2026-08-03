import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('signing out from a pushed profile reveals sign in',
      (tester) async {
    final session = sampleSession();
    final ownPost = samplePost(creator: session.profile);
    final details = ProfileDetails(
      profile: session.profile,
      posts: [ownPost],
      statistics: ProfileStatistics(totalLikes: 0, followingCount: 0),
      relationship: ProfileRelationship(
        isFollowing: false,
        isBlocked: false,
        isCurrentUser: true,
      ),
    );
    final dependencies = buildFakeDependencies(
      session: session,
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [ownPost],
        feed: FakeFeedScenario(profiles: {session.profile.id: details}),
      ),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    await tester.tap(find.widgetWithText(FilledButton, 'Profile'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Sign out'));
    await tester.pumpAndSettle();

    expect(find.text('Import your Nostr key'), findsOneWidget);
    expect(find.text('Sign out'), findsNothing);
  });
}
