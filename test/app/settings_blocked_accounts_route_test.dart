import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('opens the blocked accounts screen from settings',
      (tester) async {
    final session = sampleSession();
    final dependencies = buildFakeDependencies(
      session: session,
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [samplePost()],
        feed: FakeFeedScenario(profiles: {
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
        }),
      ),
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.person_rounded));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Open settings'));
    await tester.pumpAndSettle();
    await tester.scrollUntilVisible(
      find.text('Manage blocked accounts'),
      300,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.ensureVisible(find.text('Manage blocked accounts'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Manage blocked accounts'));
    await tester.pumpAndSettle();

    expect(find.widgetWithText(AppBar, 'Blocked accounts'), findsOneWidget);
  });
}
