import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_account_generator.dart';
import '../support/fake_profile_image_services.dart';

void main() {
  test('account picture selection failure is safe and retryable', () async {
    final account = accountCreationAccount();
    final picker = FakeProfileImagePicker()
      ..failure = const AppFailure('Photo library access was denied.');
    final cubit = AccountCreationCubit(
      FakeNostrAccountGenerator(account),
      RecordingSessionRepository(account.identity),
      RecordingProfileRepository(),
      fakeProfileImages(picker: picker),
    );
    addTearDown(cubit.close);

    await cubit.selectPicture();

    final failed = cubit.state as AccountCreationIdle;
    expect(failed.message, 'Photo library access was denied.');
    picker
      ..failure = null
      ..result = sampleProfileImage();
    await cubit.selectPicture();
    final recovered = cubit.state as AccountCreationIdle;
    expect(recovered.selectedPicture, same(picker.result));
  });
}
