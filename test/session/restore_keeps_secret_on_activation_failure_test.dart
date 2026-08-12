import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/data/secure_session_repository.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';

import '../support/fake_nostr_session_port.dart';
import '../support/memory_secret_store.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('a failed engine activation keeps the stored secret for a retry',
      () async {
    final store = MemorySecretStore()..value = testNsec;
    final port = _FlakySessionPort();
    final repository = SecureSessionRepository(
      store,
      const NdkNostrIdentityDeriver(),
      port,
    );

    await expectLater(repository.restore(), throwsA(isA<AppFailure>()));
    expect(store.value, testNsec);

    port.failNextActivation = false;
    expect((await repository.restore())?.profile.handle, startsWith('@npub'));
  });
}

class _FlakySessionPort extends FakeNostrSessionPort {
  bool failNextActivation = true;

  @override
  Future<void> activate(AuthSecret secret, NostrIdentity identity) async {
    if (failNextActivation) {
      throw const AppFailure('The Nostr engine is not ready yet.');
    }
    await super.activate(secret, identity);
  }
}
