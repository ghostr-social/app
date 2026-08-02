import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('rejects an npub that does not encode the public key', () {
    expect(
      () => NostrIdentity.parse(
        publicKeyHex: 'f' * 64,
        npub: testViewerNpub,
      ),
      throwsFormatException,
    );
  });
}
