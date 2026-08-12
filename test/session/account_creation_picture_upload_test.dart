import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_account_generator.dart';
import '../support/fake_profile_image_services.dart';

void main() {
  test(
    'signs in before uploading and publishes the uploaded picture URL',
    () async {
      final calls = <String>[];
      final account = accountCreationAccount();
      final generator = FakeNostrAccountGenerator(account);
      final sessions = RecordingSessionRepository(
        account.identity,
        calls: calls,
      );
      final profiles = RecordingProfileRepository(calls: calls);
      final picker = FakeProfileImagePicker()..result = sampleProfileImage();
      final uploader = FakeProfileImageUploader(calls: calls);
      final cubit = AccountCreationCubit(
        generator,
        sessions,
        profiles,
        fakeProfileImages(picker: picker, uploader: uploader),
      );
      addTearDown(cubit.close);

      await cubit.selectPicture();
      await cubit.begin(accountCreationMetadata());
      await cubit.complete();

      expect(calls, ['signIn', 'uploadProfilePicture', 'saveProfile']);
      expect(generator.generationCount, 1);
      expect(uploader.uploaded, same(picker.result));
      expect(profiles.savedMetadata?.pictureUrl?.value, uploader.url);
    },
  );
}
