import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/presentation/profile_edit_cubit.dart';

import '../support/fake_profile_image_services.dart';
import '../support/fake_profile_metadata_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('unexpected profile picture failures use a safe message', () async {
    final picker = FakeProfileImagePicker()..failure = StateError('private');
    final cubit = ProfileEditCubit(
      FakeProfileMetadataRepository(),
      sampleSession().identity,
      fakeProfileImages(picker: picker),
    );
    addTearDown(cubit.close);

    await cubit.selectPicture();
    expect(cubit.state.message, 'Could not save this profile.');

    picker
      ..failure = null
      ..result = sampleProfileImage();
    await cubit.selectPicture();
    final uploader = FakeProfileImageUploader()
      ..failure = StateError('private upload');
    final uploadCubit = ProfileEditCubit(
      FakeProfileMetadataRepository(),
      sampleSession().identity,
      fakeProfileImages(picker: picker, uploader: uploader),
    );
    addTearDown(uploadCubit.close);
    await uploadCubit.selectPicture();
    await uploadCubit.save(
      ProfileMetadata.parse(displayName: 'Nora', handle: 'nora'),
    );
    expect(uploadCubit.state.message, 'Could not save this profile.');
  });
}
