import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/presentation/private_key_backup_screen.dart';

import '../support/nostr_test_values.dart';

void main() {
  testWidgets('copies the generated private key only after explicit action', (
    tester,
  ) async {
    var copyCount = 0;
    await tester.pumpWidget(
      MaterialApp(
        home: PrivateKeyBackupScreen(
          secret: AuthSecret.parse(testNsec),
          onCopy: () async => copyCount += 1,
          onFinish: () {},
        ),
      ),
    );

    expect(copyCount, 0);

    await tester.tap(find.text('Copy private key'));

    expect(copyCount, 1);
  });
}
