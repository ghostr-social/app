import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/presentation/private_key_backup_screen.dart';

import '../support/nostr_test_values.dart';

void main() {
  testWidgets('hides unexpected clipboard details behind a safe message', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: PrivateKeyBackupScreen(
          secret: AuthSecret.parse(testNsec),
          onCopy: () => throw StateError('platform details'),
          onFinish: () {},
        ),
      ),
    );

    await tester.tap(find.text('Copy private key'));
    await tester.pump();

    expect(find.text('Could not copy the private key.'), findsOneWidget);
    expect(find.textContaining('platform details'), findsNothing);
  });
}
