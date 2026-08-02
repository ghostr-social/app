import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/data/secure_session_repository.dart';

import '../support/fake_nostr_session_port.dart';
import '../support/memory_secret_store.dart';

void main() {
  test('keeps the signer active when secure clearing fails', () async {
    final runtime = FakeNostrSessionPort();
    final repository = SecureSessionRepository(
      MemorySecretStore(clearError: StateError('keychain unavailable')),
      const NdkNostrIdentityDeriver(),
      runtime,
    );

    await expectLater(repository.signOut(), throwsStateError);

    expect(runtime.deactivationCount, 0);
  });
}
