import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/presentation/private_key_backup_screen.dart';

import '../support/nostr_test_values.dart';

void main() {
  testWidgets('enables Finish only after backup confirmation', (tester) async {
    var finishCount = 0;
    await tester.pumpWidget(
      MaterialApp(
        home: PrivateKeyBackupScreen(
          secret: AuthSecret.parse(testNsec),
          onFinish: () => finishCount += 1,
        ),
      ),
    );
    final finish = find.byKey(const Key('backup-finish'));

    expect(find.text('I saved my private key'), findsOneWidget);
    expect(tester.widget<ElevatedButton>(finish).onPressed, isNull);

    await tester.tap(find.byKey(const Key('backup-confirmation')));
    await tester.pump();
    expect(tester.widget<ElevatedButton>(finish).onPressed, isNotNull);

    await tester.tap(finish);
    expect(finishCount, 1);
  });
}
