import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_account_generator.dart';
import '../support/fakes.dart';
import '../support/nostr_test_values.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('pending key import locks all existing-key interactions', (
    tester,
  ) async {
    final sessions = _PendingSessionRepository();
    final generator = FakeNostrAccountGenerator(accountCreationAccount());
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      overrides: FakeDependencyOverrides(
        sessionRepository: sessions,
        accountGenerator: generator,
      ),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Use an existing key'));
    await tester.pumpAndSettle();

    final field = find.byKey(const Key('existing-key-nsec-field'));
    await tester.enterText(field, testNsec);
    await tester.tap(find.text('Continue'));
    await tester.pump();

    expect(tester.widget<TextField>(field).enabled, isFalse);
    expect(
      tester.widget<ElevatedButton>(find.byType(ElevatedButton)).onPressed,
      isNull,
    );
    expect(
      tester
          .widget<IconButton>(
            find.ancestor(
              of: find.byTooltip('Back'),
              matching: find.byType(IconButton),
            ),
          )
          .onPressed,
      isNull,
    );
    await tester.tap(find.byTooltip('Back'), warnIfMissed: false);
    await tester.tap(find.text('Signing in…'), warnIfMissed: false);
    await tester.pump();
    expect(find.text('Import your Nostr key'), findsOneWidget);
    expect(find.text('Create a Nostr account'), findsNothing);
    expect(sessions.signInAttempts, 1);
    expect(generator.generationCount, 0);
  });
}

final class _PendingSessionRepository extends FakeSessionRepository {
  final pending = Completer<UserSession>();
  int signInAttempts = 0;

  @override
  Future<UserSession> signIn(AuthSecret secret) {
    signInAttempts += 1;
    return pending.future;
  }
}
