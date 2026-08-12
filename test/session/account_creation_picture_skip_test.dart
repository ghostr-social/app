import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_nostr_account_generator.dart';
import '../support/fake_profile_image_services.dart';

void main() {
  test(
    'skipping failed picture retries with the same generated account',
    () async {
      final account = accountCreationAccount();
      final metadata = accountCreationMetadata();
      final generator = FakeNostrAccountGenerator(account);
      final provisioning = FakeAccountProvisioningRepository();
      final profiles = RecordingProfileRepository();
      final picker = FakeProfileImagePicker()..result = sampleProfileImage();
      final uploader = FakeProfileImageUploader()
        ..failure = const AppFailure('Blossom unavailable.');
      final cubit = AccountCreationCubit(
        generator,
        provisioning,
        profiles,
        fakeProfileImages(picker: picker, uploader: uploader),
      );

      await cubit.selectPicture();
      await cubit.begin(metadata);
      await cubit.complete();
      cubit.skipPicture();
      expect(
        cubit.state,
        isA<AccountCreationAwaitingBackup>()
            .having((state) => state.account, 'account', same(account))
            .having((state) => state.metadata, 'metadata', same(metadata))
            .having((state) => state.selectedPicture, 'picture', isNull),
      );
      await cubit.complete();

      expect(generator.generationCount, 1);
      expect(provisioning.stageCount, 1);
      expect(provisioning.activateCount, 2);
      expect(provisioning.discardCount, 0);
      expect(profiles.savedMetadata, same(metadata));
      expect(cubit.state, isA<AccountCreationCompleted>());
      await cubit.close();
    },
  );
}
