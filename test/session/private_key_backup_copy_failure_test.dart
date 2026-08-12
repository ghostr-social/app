import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/presentation/private_key_backup_screen.dart';

import '../support/nostr_test_values.dart';

void main() {
  testWidgets('reports failed key copy and blocks finish while copying', (
    tester,
  ) async {
    final pending = Completer<void>();
    await tester.pumpWidget(
      MaterialApp(
        home: PrivateKeyBackupScreen(
          secret: AuthSecret.parse(testNsec),
          onCopy: () => pending.future,
          onFinish: () {},
        ),
      ),
    );
    await tester.tap(find.byKey(const Key('backup-confirmation')));
    await tester.tap(find.text('Copy private key'));
    await tester.pump();

    expect(_finish(tester).onPressed, isNull);

    pending.completeError(const AppFailure('Could not copy the private key.'));
    await tester.pump();

    expect(find.text('Could not copy the private key.'), findsOneWidget);
    expect(_finish(tester).onPressed, isNotNull);
  });
}

ElevatedButton _finish(WidgetTester tester) {
  return tester.widget(find.byKey(const Key('backup-finish')));
}
