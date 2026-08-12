import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/profile/presentation/profile_edit_cubit.dart';

import '../support/fake_profile_image_services.dart';
import '../support/fake_profile_metadata_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('profile edit reports picker denial and remains retryable', () async {
    final picker = FakeProfileImagePicker()
      ..failure = const AppFailure('Photo library access was denied.');
    final cubit = ProfileEditCubit(
      FakeProfileMetadataRepository(),
      sampleSession().identity,
      fakeProfileImages(picker: picker),
    );
    addTearDown(cubit.close);

    await cubit.selectPicture();

    expect(cubit.state, isA<ProfileEditFailure>());
    expect(cubit.state.message, 'Photo library access was denied.');
    picker
      ..failure = null
      ..result = sampleProfileImage();
    await cubit.selectPicture();
    expect(cubit.state.selectedPicture, same(picker.result));
  });
}
