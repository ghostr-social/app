import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('long-lived user sessions do not retain the private key', () {
    final source = File(
      'lib/features/session/domain/user_session.dart',
    ).readAsStringSync();

    expect(source, isNot(contains('AuthSecret')));
    expect(source, isNot(contains('secret')));
  });
}
