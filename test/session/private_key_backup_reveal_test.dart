import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/presentation/private_key_backup_screen.dart';

import '../support/nostr_test_values.dart';

void main() {
  testWidgets('keeps the generated nsec hidden until explicitly revealed', (
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
    final secretField = find.byKey(const Key('backup-private-key-field'));
    final hidden = tester.widget<TextField>(secretField);

    expect(hidden.controller?.text, testNsec);
    expect(hidden.readOnly, isTrue);
    expect(hidden.obscureText, isTrue);

    await tester.tap(find.byTooltip('Reveal private key'));
    await tester.pump();

    expect(tester.widget<TextField>(secretField).obscureText, isFalse);
    expect(find.byTooltip('Hide private key'), findsOneWidget);
  });
}
