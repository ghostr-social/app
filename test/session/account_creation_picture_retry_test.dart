import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_account_generator.dart';
import '../support/fake_profile_image_services.dart';

void main() {
  test(
    'upload retry keeps the same generated private key and picture',
    () async {
      final account = accountCreationAccount();
      final generator = FakeNostrAccountGenerator(account);
      final picker = FakeProfileImagePicker()..result = sampleProfileImage();
      final uploader = FakeProfileImageUploader()
        ..failure = const AppFailure('Blossom rejected the picture.');
      final cubit = AccountCreationCubit(
        generator,
        RecordingSessionRepository(account.identity),
        RecordingProfileRepository(),
        fakeProfileImages(picker: picker, uploader: uploader),
      );
      addTearDown(cubit.close);

      await cubit.selectPicture();
      await cubit.begin(accountCreationMetadata());
      await cubit.complete();
      expect(
        (cubit.state as AccountCreationFailure).message,
        'Blossom rejected the picture.',
      );

      uploader.failure = null;
      await cubit.complete();

      expect(generator.generationCount, 1);
      expect(uploader.uploadCount, 2);
      expect(cubit.state, isA<AccountCreationCompleted>());
    },
  );
}
