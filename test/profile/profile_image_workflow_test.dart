import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';

import '../support/fake_profile_image_services.dart';

void main() {
  test(
    'selected image upload replaces a manually entered picture URL',
    () async {
      final picker = FakeProfileImagePicker()..result = sampleProfileImage();
      final uploader = FakeProfileImageUploader();
      final workflow = fakeProfileImages(picker: picker, uploader: uploader);
      final metadata = ProfileMetadata.parse(
        displayName: 'Nora',
        handle: 'nora',
        pictureUrl: 'https://old.example/avatar.jpg',
      );

      final selected = await workflow.select();
      final resolved = await workflow.resolve(metadata, selected);

      expect(selected, same(picker.result));
      expect(uploader.uploaded, same(selected));
      expect(resolved.pictureUrl?.value, uploader.url);
    },
  );
}
