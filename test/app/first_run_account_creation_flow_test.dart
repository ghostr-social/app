import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/generated_nostr_account.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';

import '../support/fake_nostr_account_generator.dart';
import '../support/fake_profile_metadata_repository.dart';
import '../support/fakes.dart';
import '../support/nostr_test_values.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('new user creates and backs up a profiled Nostr account', (
    tester,
  ) async {
    final account = GeneratedNostrAccount(
      secret: AuthSecret.parse(testNsec),
      identity: NostrIdentity.parse(
        publicKeyHex: testViewerPublicKey,
        npub: testViewerNpub,
      ),
    );
    final generator = FakeNostrAccountGenerator(account);
    final profiles = FakeProfileMetadataRepository();
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      accountGenerator: generator,
      profileMetadataRepository: profiles,
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    expect(find.text('Create a Nostr account'), findsOneWidget);
    expect(find.text('Use an existing key'), findsOneWidget);

    await tester.tap(find.text('Create a Nostr account'));
    await tester.pumpAndSettle();
    await tester.enterText(
      find.byKey(const Key('profile-display-name-field')),
      'Nora Relay',
    );
    await tester.enterText(
      find.byKey(const Key('profile-handle-field')),
      '@nora',
    );
    await tester.pump();
    final create = find.byKey(const Key('create-account-submit'));
    expect(tester.widget<ElevatedButton>(create).onPressed, isNotNull);
    await tester.tap(find.text('Create account'));
    await tester.pumpAndSettle();

    expect(generator.generationCount, 1);
    expect(find.text('Back up your private key'), findsOneWidget);
    expect(find.textContaining('no password reset'), findsOneWidget);
    final finish = tester.widget<ElevatedButton>(
      find.widgetWithText(ElevatedButton, 'Finish'),
    );
    expect(finish.onPressed, isNull);

    await tester.tap(find.byKey(const Key('backup-confirmation')));
    await tester.pump();
    await tester.tap(find.text('Finish'));
    await tester.pumpAndSettle();

    expect(profiles.savedMetadata?.displayName.value, 'Nora Relay');
    expect(profiles.savedMetadata?.handle.value, 'nora');
    expect(find.text('Home'), findsOneWidget);
    expect(find.text('Profile'), findsOneWidget);
  });
}
