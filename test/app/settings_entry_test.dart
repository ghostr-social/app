import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('opens relay and video inventory settings from own profile',
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

    expect(find.text('Settings'), findsOneWidget);
    expect(find.text('Relay connections'), findsOneWidget);
    expect(find.text('Video inventory'), findsOneWidget);
  });
}
