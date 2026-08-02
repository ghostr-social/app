import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/data/secure_session_repository.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/session/domain/nostr_session_port.dart';

import '../support/memory_secret_store.dart';

void main() {
  test('clears a persisted nsec when signer activation fails', () async {
    final store = MemorySecretStore();
    final repository = SecureSessionRepository(
      store,
      const NdkNostrIdentityDeriver(),
      _FailingSessionPort(),
    );
    final secret = AuthSecret.parse(
      'nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5',
    );

    await expectLater(repository.signIn(secret), throwsStateError);

    expect(await store.read(), isNull);
  });
}

class _FailingSessionPort implements NostrSessionPort {
  @override
  void activate(AuthSecret secret, NostrIdentity identity) {
    throw StateError('signer unavailable');
  }

  @override
  void deactivate() {}
}
