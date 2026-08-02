import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/data/secure_session_repository.dart';

import '../support/fake_nostr_session_port.dart';
import '../support/memory_secret_store.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('clears a corrupt stored session without reading it first', () async {
    final store = MemorySecretStore(readError: StateError('corrupt entry'))
      ..value = testNsec;
    final nostrSession = FakeNostrSessionPort();
    final repository = SecureSessionRepository(
      store,
      const NdkNostrIdentityDeriver(),
      nostrSession,
    );

    await repository.resetStoredSession();

    expect(store.value, isNull);
    expect(nostrSession.deactivationCount, 1);
  });
}
