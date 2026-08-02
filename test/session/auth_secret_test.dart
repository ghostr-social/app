import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('accepts nsec-style secrets and rejects invalid input', () {
    expect(AuthSecret.tryParse(testNsec), isNotNull);
    expect(AuthSecret.tryParse('nsec1validghostrsecretvalue123456'), isNull);
    expect(AuthSecret.tryParse(''), isNull);
    expect(AuthSecret.tryParse('npub1notasecret'), isNull);
  });
}
