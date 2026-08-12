import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_account_generator.dart';
import '../support/fake_profile_image_services.dart';

void main() {
  test('unexpected picture failures never expose plugin details', () async {
    final account = accountCreationAccount();
    final picker = FakeProfileImagePicker()..failure = StateError('private');
    final uploader = FakeProfileImageUploader();
    final cubit = AccountCreationCubit(
      FakeNostrAccountGenerator(account),
      RecordingSessionRepository(account.identity),
      RecordingProfileRepository(),
      fakeProfileImages(picker: picker, uploader: uploader),
    );
    addTearDown(cubit.close);

    await cubit.selectPicture();
    expect(
      (cubit.state as AccountCreationIdle).message,
      'Could not select the profile picture.',
    );

    picker
      ..failure = null
      ..result = sampleProfileImage();
    uploader.failure = StateError('private upload');
    await cubit.selectPicture();
    await cubit.begin(accountCreationMetadata());
    await cubit.complete();
    expect(
      (cubit.state as AccountCreationFailure).message,
      'Could not finish creating this account.',
    );
  });
}
