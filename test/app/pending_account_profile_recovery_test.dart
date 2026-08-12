import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_nostr_account_generator.dart';
import '../support/fakes.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('recovered key requires profile and backup before activation', (
    tester,
  ) async {
    final account = accountCreationAccount();
    final generator = FakeNostrAccountGenerator(account);
    final provisioning = FakeAccountProvisioningRepository()
      ..pending = PendingAccountProfileRecovery(account);
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      accountGenerator: generator,
      accountProvisioningRepository: provisioning,
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    expect(find.text('Create your profile'), findsOneWidget);
    expect(find.text('Back up your private key'), findsNothing);
    expect(provisioning.activateCount, 0);
    final submit = find.byKey(const Key('create-account-submit'));
    expect(tester.widget<ElevatedButton>(submit).onPressed, isNull);

    await tester.enterText(
      find.byKey(const Key('profile-display-name-field')),
      'Recovered Nora',
    );
    await tester.enterText(
      find.byKey(const Key('profile-handle-field')),
      '@recovered_nora',
    );
    await tester.pump();
    await tester.tap(submit);
    await tester.pumpAndSettle();

    expect(find.text('Back up your private key'), findsOneWidget);
    expect(generator.generationCount, 0);
    expect(provisioning.activateCount, 0);
    expect(provisioning.pending?.account, same(account));
    final finish = find.byKey(const Key('backup-finish'));
    expect(tester.widget<ElevatedButton>(finish).onPressed, isNull);

    await tester.tap(find.byKey(const Key('backup-confirmation')));
    await tester.pump();
    await tester.tap(finish);
    await tester.pumpAndSettle();

    expect(provisioning.activateCount, 1);
    expect(find.text('Home'), findsOneWidget);
  });
}
