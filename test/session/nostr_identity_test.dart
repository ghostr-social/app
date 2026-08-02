import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/secure_session_repository.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';

import '../support/memory_secret_store.dart';
import '../support/fake_nostr_session_port.dart';

void main() {
  test('imports an nsec as its real Nostr public identity', () async {
    const nsec =
        'nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5';
    const npub =
        'npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg';
    final repository = SecureSessionRepository(
      MemorySecretStore(),
      NdkNostrIdentityDeriver(),
      FakeNostrSessionPort(),
    );

    final session = await repository.signIn(AuthSecret.parse(nsec));

    expect(session.profile.id, npub);
    expect(session.profile.handle, '@$npub');
  });
}
