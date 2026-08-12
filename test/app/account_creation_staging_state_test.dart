import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_nostr_account_generator.dart';
import '../support/fakes.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('account creation disables the profile form while securing key', (
    tester,
  ) async {
    final gate = Completer<void>();
    final provisioning = FakeAccountProvisioningRepository()..stageGate = gate;
    final account = accountCreationAccount();
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      accountGenerator: FakeNostrAccountGenerator(account),
      accountProvisioningRepository: provisioning,
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
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
    await tester.tap(find.text('Create account'));
    await tester.pump();
    expect(find.text('Saving…'), findsOneWidget);
    expect(
      tester
          .widget<ElevatedButton>(
            find.byKey(const Key('create-account-submit')),
          )
          .onPressed,
      isNull,
    );
    expect(
      tester
          .widget<IconButton>(
            find.ancestor(
              of: find.byIcon(Icons.arrow_back),
              matching: find.byType(IconButton),
            ),
          )
          .onPressed,
      isNull,
    );
    gate.complete();
    await tester.pumpAndSettle();
    expect(find.text('Back up your private key'), findsOneWidget);
  });
}
