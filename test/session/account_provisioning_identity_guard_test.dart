import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/local_account_provisioning_repository.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/domain/generated_nostr_account.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/memory_secret_store.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('activation rejects an identity that does not match its key', () async {
    SharedPreferences.setMockInitialValues({});
    final original = accountCreationAccount();
    final setup = PendingAccountSetup(
      account: GeneratedNostrAccount(
        secret: original.secret,
        identity: NostrIdentity.parse(
          publicKeyHex: testCreatorPublicKey,
          npub: testCreatorNpub,
        ),
      ),
      metadata: accountCreationMetadata(),
    );
    final repository = LocalAccountProvisioningRepository(
      await SharedPreferences.getInstance(),
      AccountProvisioningSecretStores(
        pending: MemorySecretStore(),
        active: MemorySecretStore(),
      ),
      const NdkNostrIdentityDeriver(),
      FakeNostrSessionPort(),
    );

    await expectLater(repository.activate(setup), throwsStateError);
  });
}
