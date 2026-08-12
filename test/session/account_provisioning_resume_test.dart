import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/local_account_provisioning_repository.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/fake_profile_image_services.dart';
import '../support/memory_secret_store.dart';
import '../support/nostr_test_values.dart';

void main() {
  test(
    'pending account survives restart and commits only after setup',
    () async {
      SharedPreferences.setMockInitialValues({});
      final preferences = await SharedPreferences.getInstance();
      final pendingSecrets = MemorySecretStore();
      final activeSecrets = MemorySecretStore();
      final nostr = FakeNostrSessionPort();
      final setup = PendingAccountSetup(
        account: accountCreationAccount(),
        metadata: accountCreationMetadata(),
        selectedPicture: sampleProfileImage(),
      );
      final repository = LocalAccountProvisioningRepository(
        preferences,
        AccountProvisioningSecretStores(
          pending: pendingSecrets,
          active: activeSecrets,
        ),
        const NdkNostrIdentityDeriver(),
        nostr,
      );

      await repository.stage(setup);
      expect(activeSecrets.value, isNull);
      expect(preferences.getKeys(), isNotEmpty);
      final publicDraft = preferences.getString(preferences.getKeys().single)!;
      expect(publicDraft, isNot(contains(testNsec)));
      expect(publicDraft, isNot(contains(sampleProfileImage().path)));

      final restored = await repository.restorePending();
      expect(restored, isA<PendingAccountSetup>());
      final restoredSetup = restored! as PendingAccountSetup;
      expect(restoredSetup.account.secret.value, testNsec);
      expect(restoredSetup.metadata.handle.value, 'nora');
      expect(restoredSetup.selectedPicture, isNull);
      await repository.activate(restoredSetup);
      expect(activeSecrets.value, isNull);

      await repository.commit(restoredSetup);
      expect(activeSecrets.value, testNsec);
      expect(await repository.restorePending(), isNull);
    },
  );
}
