import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/secure_session_repository.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';

import '../support/memory_secret_store.dart';
import '../support/fake_nostr_session_port.dart';

void main() {
  test('persists, restores, and clears the viewer secret', () async {
    final store = MemorySecretStore();
    final repository = SecureSessionRepository(
      store,
      const NdkNostrIdentityDeriver(),
      FakeNostrSessionPort(),
    );

    await repository.signIn(AuthSecret.parse(
      'nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5',
    ));
    expect((await repository.restore())?.profile.handle, startsWith('@npub'));
    await repository.signOut();
    expect(await repository.restore(), isNull);
  });
}
