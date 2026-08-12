import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/router/app_router.dart';
import 'package:ghostr/app/profile_route_request.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('forwards sign out from a routed current-user profile', (
    tester,
  ) async {
    var signedOut = false;
    final session = sampleSession();
    final details = ProfileDetails(
      profile: session.profile,
      posts: const [],
      statistics: ProfileStatistics(totalLikes: 0, followingCount: 0),
      relationship: ProfileRelationship(
        isFollowing: false,
        isBlocked: false,
        isCurrentUser: true,
      ),
    );
    final catalog = FakeVideoCatalogRepository(
      forYouFeed: [],
      feed: FakeFeedScenario(profiles: {session.profile.id: details}),
    );
    final controllers = AppControllerFactory(
      buildFakeDependencies(session: session, catalogRepository: catalog),
    );
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) {
            return ElevatedButton(
              onPressed: () => Navigator.of(context).push(
                AppRouter.profile(
                  ProfileRouteRequest(
                    session: session,
                    profileId: session.profile.id,
                    controllers: controllers,
                    onSignedOut: () => signedOut = true,
                  ),
                ),
              ),
              child: const Text('Open own profile'),
            );
          },
        ),
      ),
    );

    await tester.tap(find.text('Open own profile'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Sign out'));

    expect(signedOut, isTrue);
  });
}
