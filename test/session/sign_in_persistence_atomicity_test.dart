import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/data/secure_session_repository.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';

import '../support/fake_nostr_session_port.dart';
import '../support/memory_secret_store.dart';

void main() {
  test('does not activate a signer when secure persistence fails', () async {
    final runtime = FakeNostrSessionPort();
    final repository = SecureSessionRepository(
      MemorySecretStore(writeError: StateError('keychain unavailable')),
      const NdkNostrIdentityDeriver(),
      runtime,
    );
    final secret = AuthSecret.parse(
      'nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5',
    );

    await expectLater(repository.signIn(secret), throwsStateError);

    expect(runtime.activationCount, 0);
  });
}
