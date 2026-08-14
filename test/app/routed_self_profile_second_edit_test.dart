import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/router/app_router.dart';
import 'package:ghostr/app/profile_route_request.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

import '../support/fake_profile_metadata_repository.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('second routed self edit starts from the latest profile', (
    tester,
  ) async {
    final session = sampleSession();
    final metadata = FakeProfileMetadataRepository();
    final post = samplePost(creator: session.profile);
    final details = ProfileDetails(
      profile: session.profile,
      posts: [post],
      statistics: ProfileStatistics(totalLikes: 0, followingCount: 0),
      relationship: ProfileRelationship(
        isFollowing: false,
        isBlocked: false,
        isCurrentUser: true,
      ),
    );
    final dependencies = buildFakeDependencies(
      session: session,
      overrides: FakeDependencyOverrides(profileMetadataRepository: metadata),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [post],
        feed: FakeFeedScenario(profiles: {session.profile.id: details}),
      ),
    );
    final controllers = AppControllerFactory(dependencies);
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) => ElevatedButton(
            onPressed: () => Navigator.of(context).push(
              AppRouter.profile(
                ProfileRouteRequest(
                  session: session,
                  profileId: session.profile.id,
                  controllers: controllers,
                  onSignedOut: () {},
                ),
              ),
            ),
            child: const Text('Open own profile'),
          ),
        ),
      ),
    );
    await tester.tap(find.text('Open own profile'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Edit profile'));
    await tester.pumpAndSettle();
    await tester.enterText(
      find.byKey(const Key('profile-display-name-field')),
      'Latest Nora',
    );
    await tester.enterText(
      find.byKey(const Key('profile-handle-field')),
      '@latest_nora',
    );
    await tester.tap(find.text('Save profile'));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('profile-display-name-field')), findsNothing);

    await tester.tap(find.text('Edit profile'));
    await tester.pumpAndSettle();
    final name = tester.widget<TextField>(
      find.byKey(const Key('profile-display-name-field')),
    );
    expect(name.controller?.text, 'Latest Nora');
  });
}
