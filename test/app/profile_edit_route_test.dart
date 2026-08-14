import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

import '../support/fake_profile_metadata_repository.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('current user edits name, handle, and picture URL', (
    tester,
  ) async {
    final session = sampleSession();
    final profiles = FakeProfileMetadataRepository();
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
    final dependencies = buildFakeDependencies(
      session: session,
      overrides: FakeDependencyOverrides(profileMetadataRepository: profiles),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: const [],
        feed: FakeFeedScenario(profiles: {session.profile.id: details}),
      ),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.person_rounded));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Edit profile'));
    await tester.pumpAndSettle();
    await tester.enterText(
      find.byKey(const Key('profile-display-name-field')),
      'Nora Updated',
    );
    await tester.enterText(
      find.byKey(const Key('profile-handle-field')),
      '@nora_updated',
    );
    await tester.enterText(
      find.byKey(const Key('profile-picture-url-field')),
      'https://cdn.example/nora.png',
    );
    await tester.tap(find.text('Save profile'));
    await tester.pumpAndSettle();

    expect(profiles.savedMetadata?.handle.value, 'nora_updated');
    expect(find.text('Nora Updated'), findsOneWidget);
    expect(find.text('@nora_updated'), findsOneWidget);
  });
}
