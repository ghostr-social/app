import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/presentation/private_key_backup_screen.dart';

import '../support/nostr_test_values.dart';

void main() {
  testWidgets('shows a safe account creation error beside backup controls', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: PrivateKeyBackupScreen(
          secret: AuthSecret.parse(testNsec),
          errorMessage: 'Could not publish your profile.',
          onFinish: () {},
        ),
      ),
    );

    final error = tester.widget<Text>(
      find.text('Could not publish your profile.'),
    );
    final context = tester.element(find.byType(PrivateKeyBackupScreen));

    expect(error.style?.color, Theme.of(context).colorScheme.error);
    expect(find.text('I saved my private key'), findsOneWidget);
  });
}
