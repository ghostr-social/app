import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/platform/session/system_secret_clipboard.dart';

import '../support/nostr_test_values.dart';

void main() {
  test(
    'copies only the recovery secret value to the platform clipboard',
    () async {
      ClipboardData? written;
      final clipboard = SystemSecretClipboard(
        writer: (data) async => written = data,
      );

      await clipboard.copy(AuthSecret.parse(testNsec));

      expect(written?.text, testNsec);
    },
  );
}
