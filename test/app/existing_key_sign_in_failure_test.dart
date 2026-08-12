import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

import '../support/fakes.dart';
import '../support/nostr_test_values.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('failed existing-key sign in stays visible and can retry', (
    tester,
  ) async {
    final sessions = _RetryingSessionRepository();
    final dependencies = buildFakeDependencies(
      sessionRepository: sessions,
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Use an existing key'));
    await tester.pumpAndSettle();

    final field = find.byKey(const Key('existing-key-nsec-field'));
    await tester.enterText(field, testNsec);
    await tester.tap(find.text('Continue'));
    await tester.pump();
    sessions.rejectFirstAttempt();
    await tester.pumpAndSettle();

    expect(find.text('Import your Nostr key'), findsOneWidget);
    expect(find.text('Relay activation failed.'), findsOneWidget);
    expect(tester.widget<TextField>(field).controller?.text, testNsec);
    await tester.tap(find.text('Continue'));
    await tester.pumpAndSettle();
    expect(sessions.signInAttempts, 2);
    expect(find.text('Home'), findsOneWidget);
  });
}

final class _RetryingSessionRepository extends FakeSessionRepository {
  int signInAttempts = 0;
  Completer<UserSession>? _firstAttempt;

  @override
  Future<UserSession> signIn(AuthSecret secret) {
    signInAttempts += 1;
    if (signInAttempts == 1) {
      _firstAttempt = Completer<UserSession>();
      return _firstAttempt!.future;
    }
    return super.signIn(secret);
  }

  void rejectFirstAttempt() => _firstAttempt!.completeError(
    const AppFailure('Relay activation failed.'),
  );
}
