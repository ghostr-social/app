import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/local_account_provisioning_repository.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_account_generator.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/memory_secret_store.dart';

void main() {
  test('failed generated account resumes after cubit recreation', () async {
    SharedPreferences.setMockInitialValues({});
    final account = accountCreationAccount();
    final profiles = RecordingProfileRepository();
    final activeSecrets = MemorySecretStore();
    final repository = LocalAccountProvisioningRepository(
      await SharedPreferences.getInstance(),
      AccountProvisioningSecretStores(
        pending: MemorySecretStore(),
        active: activeSecrets,
      ),
      const NdkNostrIdentityDeriver(),
      FakeNostrSessionPort(),
    );
    final first = AccountCreationCubit(
      FakeNostrAccountGenerator(account),
      repository,
      profiles,
    );

    await first.begin(accountCreationMetadata());
    expect(first.state, isA<AccountCreationAwaitingBackup>());
    expect(activeSecrets.value, isNull);
    await first.close();

    final resumed = AccountCreationCubit(
      FakeNostrAccountGenerator(account),
      repository,
      profiles,
    );
    await resumed.restorePending();
    final pending = resumed.state as AccountCreationAwaitingBackup;
    expect(pending.account.secret.value, account.secret.value);
    await resumed.complete();

    expect(resumed.state, isA<AccountCreationCompleted>());
    expect(activeSecrets.value, account.secret.value);
    expect(await repository.restorePending(), isNull);
    await resumed.close();
  });
}
