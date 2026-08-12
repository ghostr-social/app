import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/local_account_provisioning_repository.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/memory_secret_store.dart';

void main() {
  test('failed pending-key write rolls back the public draft', () async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final repository = LocalAccountProvisioningRepository(
      preferences,
      AccountProvisioningSecretStores(
        pending: MemorySecretStore(
          writeError: StateError('secure store failed'),
        ),
        active: MemorySecretStore(),
      ),
      const NdkNostrIdentityDeriver(),
      FakeNostrSessionPort(),
    );
    final setup = PendingAccountSetup(
      account: accountCreationAccount(),
      metadata: accountCreationMetadata(),
    );

    await expectLater(repository.stage(setup), throwsStateError);

    expect(preferences.getKeys(), isEmpty);
  });
}
