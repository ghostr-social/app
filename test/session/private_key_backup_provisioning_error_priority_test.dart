import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/presentation/private_key_backup_screen.dart';

import '../support/nostr_test_values.dart';

void main() {
  testWidgets('account provisioning error supersedes an older copy error', (
    tester,
  ) async {
    String? provisioningError;
    late StateSetter rebuild;
    await tester.pumpWidget(
      MaterialApp(
        home: StatefulBuilder(
          builder: (_, setState) {
            rebuild = setState;
            return PrivateKeyBackupScreen(
              secret: AuthSecret.parse(testNsec),
              errorMessage: provisioningError,
              onCopy: () => throw const AppFailure('Copy failed.'),
              onFinish: () {},
            );
          },
        ),
      ),
    );
    await tester.tap(find.text('Copy private key'));
    await tester.pump();
    expect(find.text('Copy failed.'), findsOneWidget);

    rebuild(() => provisioningError = 'Profile publish failed.');
    await tester.pump();

    expect(find.text('Profile publish failed.'), findsOneWidget);
    expect(find.text('Copy failed.'), findsNothing);
  });
}
