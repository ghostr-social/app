import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

import '../support/fake_profile_metadata_repository.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('routed self profile edits the profile and active session', (
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
      profileMetadataRepository: metadata,
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [post],
        feed: FakeFeedScenario(profiles: {session.profile.id: details}),
      ),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Open profile'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Edit profile'));
    await tester.pumpAndSettle();
    await tester.enterText(
      find.byKey(const Key('profile-display-name-field')),
      'Routed Nora',
    );
    await tester.enterText(
      find.byKey(const Key('profile-handle-field')),
      '@routed_nora',
    );
    await tester.tap(find.text('Save profile'));
    await tester.pumpAndSettle();

    expect(metadata.savedMetadata?.displayName.value, 'Routed Nora');
    expect(find.text('Routed Nora'), findsOneWidget);
    await tester.pageBack();
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.person_rounded));
    await tester.pumpAndSettle();
    expect(find.text('Routed Nora'), findsOneWidget);
  });
}
