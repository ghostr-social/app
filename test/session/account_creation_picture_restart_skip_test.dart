import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/session/data/local_account_provisioning_repository.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_account_generator.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/fake_profile_image_services.dart';
import '../support/memory_secret_store.dart';

void main() {
  test(
    'restart resumes a failed temporary picture without the picture',
    () async {
      SharedPreferences.setMockInitialValues({});
      final account = accountCreationAccount();
      final generator = FakeNostrAccountGenerator(account);
      final profiles = RecordingProfileRepository();
      final uploader = FakeProfileImageUploader()
        ..failure = const AppFailure('Blossom unavailable.');
      final workflow = fakeProfileImages(
        picker: FakeProfileImagePicker()..result = sampleProfileImage(),
        uploader: uploader,
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
      final first = AccountCreationCubit(
        generator,
        repository,
        profiles,
        workflow,
      );
      final metadata = ProfileMetadata.parse(
        displayName: 'Nora Relay',
        handle: '@nora',
      );
      await first.selectPicture();
      await first.begin(metadata);
      await first.complete();
      expect(first.state, isA<AccountCreationFailure>());
      await first.close();

      final resumed = AccountCreationCubit(
        generator,
        repository,
        profiles,
        workflow,
      );
      await resumed.restorePending();
      expect(
        resumed.state,
        isA<AccountCreationAwaitingBackup>().having(
          (state) => state.selectedPicture,
          'temporary picture',
          isNull,
        ),
      );
      await resumed.complete();

      expect(resumed.state, isA<AccountCreationCompleted>());
      expect(generator.generationCount, 1);
      expect(uploader.uploadCount, 1);
      expect(
        profiles.savedMetadata?.displayName.value,
        metadata.displayName.value,
      );
      expect(profiles.savedMetadata?.handle.value, metadata.handle.value);
      expect(profiles.savedMetadata?.pictureUrl, isNull);
      await resumed.close();
    },
  );
}
