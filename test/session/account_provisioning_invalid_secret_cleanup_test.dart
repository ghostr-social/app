import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/local_account_provisioning_repository.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_session_port.dart';
import '../support/memory_secret_store.dart';

void main() {
  test('invalid pending secret and its public draft are discarded', () async {
    SharedPreferences.setMockInitialValues({
      'ghostr.account.provisioning.v1': '{"displayName":"Nora"}',
    });
    final preferences = await SharedPreferences.getInstance();
    final pendingSecrets = MemorySecretStore()..value = 'not-an-nsec';
    final repository = LocalAccountProvisioningRepository(
      preferences,
      AccountProvisioningSecretStores(
        pending: pendingSecrets,
        active: MemorySecretStore(),
      ),
      const NdkNostrIdentityDeriver(),
      FakeNostrSessionPort(),
    );

    expect(await repository.restorePending(), isNull);
    expect(pendingSecrets.value, isNull);
    expect(preferences.getKeys(), isEmpty);
  });
}
