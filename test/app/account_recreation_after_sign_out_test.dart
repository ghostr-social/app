import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

import '../support/fake_nostr_account_generator.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('can create another account after signing out', (tester) async {
    final session = sampleSession();
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: const [],
        feed: FakeFeedScenario(
          profiles: {session.profile.id: _currentUserProfile(session.profile)},
        ),
      ),
    );
    final generator =
        dependencies.accountGenerator as FakeNostrAccountGenerator;

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    await _startAccount(tester, name: 'First User', handle: '@first');
    await _finishAccount(tester);

    await tester.tap(find.text('Profile').last);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Sign out'));
    await tester.pumpAndSettle();

    expect(find.text('Welcome to Ghostr'), findsOneWidget);
    await _startAccount(tester, name: 'Second User', handle: '@second');

    expect(generator.generationCount, 2);
    expect(find.text('Back up your private key'), findsOneWidget);
  });
}

ProfileDetails _currentUserProfile(ProfileSummary profile) {
  return ProfileDetails(
    profile: profile,
    posts: const [],
    statistics: ProfileStatistics(totalLikes: 0, followingCount: 0),
    relationship: ProfileRelationship(
      isFollowing: false,
      isBlocked: false,
      isCurrentUser: true,
    ),
  );
}

Future<void> _startAccount(
  WidgetTester tester, {
  required String name,
  required String handle,
}) async {
  await tester.tap(find.text('Create a Nostr account'));
  await tester.pumpAndSettle();
  await tester.enterText(
    find.byKey(const Key('profile-display-name-field')),
    name,
  );
  await tester.enterText(find.byKey(const Key('profile-handle-field')), handle);
  await tester.pump();
  final submit = find.byKey(const Key('create-account-submit'));
  expect(tester.widget<ElevatedButton>(submit).onPressed, isNotNull);
  await tester.tap(find.text('Create account'));
  await tester.pumpAndSettle();
}

Future<void> _finishAccount(WidgetTester tester) async {
  await tester.tap(find.byKey(const Key('backup-confirmation')));
  await tester.pump();
  await tester.tap(find.text('Finish'));
  await tester.pumpAndSettle();
}
