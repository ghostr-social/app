import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/data/secure_session_repository.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/session/domain/nostr_session_port.dart';

import '../support/memory_secret_store.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('restores the persisted nsec when signer deactivation fails', () async {
    final store = MemorySecretStore()..value = testNsec;
    final repository = SecureSessionRepository(
      store,
      const NdkNostrIdentityDeriver(),
      _FailingSessionPort(),
    );

    await expectLater(repository.signOut(), throwsStateError);

    expect(await store.read(), testNsec);
  });
}

class _FailingSessionPort implements NostrSessionPort {
  @override
  Future<void> activate(AuthSecret secret, NostrIdentity identity) async {}

  @override
  Future<void> deactivate() async => throw StateError('signer unavailable');
}
