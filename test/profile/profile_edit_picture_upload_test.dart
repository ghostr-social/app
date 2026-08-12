import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/presentation/profile_edit_cubit.dart';

import '../support/fake_profile_image_services.dart';
import '../support/fake_profile_metadata_repository.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'profile edit uploads the selected image before saving metadata',
    () async {
      final repository = FakeProfileMetadataRepository();
      final picker = FakeProfileImagePicker()..result = sampleProfileImage();
      final uploader = FakeProfileImageUploader();
      final cubit = ProfileEditCubit(
        repository,
        sampleSession().identity,
        fakeProfileImages(picker: picker, uploader: uploader),
      );
      addTearDown(cubit.close);

      await cubit.selectPicture();
      await cubit.save(
        ProfileMetadata.parse(
          displayName: 'Nora',
          handle: 'nora',
          pictureUrl: 'https://old.example/avatar.jpg',
        ),
      );

      expect(uploader.uploaded, same(picker.result));
      expect(repository.savedMetadata?.pictureUrl?.value, uploader.url);
      expect(cubit.state, isA<ProfileEditSaved>());
    },
  );
}
