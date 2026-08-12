import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/platform/session/system_secret_clipboard.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('translates a platform clipboard failure', () async {
    final clipboard = SystemSecretClipboard(
      writer: (_) => throw StateError('clipboard unavailable'),
    );

    await expectLater(
      clipboard.copy(AuthSecret.parse(testNsec)),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Could not copy the private key.',
        ),
      ),
    );
  });
}
