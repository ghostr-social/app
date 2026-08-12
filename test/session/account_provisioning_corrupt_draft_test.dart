import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/local_account_provisioning_repository.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_session_port.dart';
import '../support/memory_secret_store.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('corrupt public draft preserves its valid secure pending key', () async {
    SharedPreferences.setMockInitialValues({
      'ghostr.account.provisioning.v1': '{"npub":7}',
    });
    final preferences = await SharedPreferences.getInstance();
    final pendingSecrets = MemorySecretStore()..value = testNsec;
    final repository = LocalAccountProvisioningRepository(
      preferences,
      AccountProvisioningSecretStores(
        pending: pendingSecrets,
        active: MemorySecretStore(),
      ),
      const NdkNostrIdentityDeriver(),
      FakeNostrSessionPort(),
    );

    final restored = await repository.restorePending();

    expect(restored, isNotNull);
    expect(restored?.account.secret.value, testNsec);
    expect(pendingSecrets.value, testNsec);
  });
}
