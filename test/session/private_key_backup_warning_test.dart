import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/presentation/private_key_backup_screen.dart';

import '../support/nostr_test_values.dart';

void main() {
  testWidgets('warns that a lost private key cannot be recovered', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: PrivateKeyBackupScreen(
          secret: AuthSecret.parse(testNsec),
          onFinish: () {},
        ),
      ),
    );

    expect(find.text('Back up your private key'), findsOneWidget);
    expect(find.textContaining('no password reset'), findsOneWidget);
    expect(find.textContaining('lose access'), findsOneWidget);
    expect(find.textContaining('public key is safe to share'), findsOneWidget);
    expect(find.textContaining('device is not a backup'), findsOneWidget);
  });
}
