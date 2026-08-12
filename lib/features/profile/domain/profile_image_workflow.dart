import 'package:ghostr/features/profile/domain/profile_image_picker_port.dart';
import 'package:ghostr/features/profile/domain/profile_image_upload_port.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';

final class ProfileImageWorkflow {
  const ProfileImageWorkflow(this._picker, this._uploader);

  const ProfileImageWorkflow.disabled()
    : _picker = const _DisabledProfileImagePicker(),
      _uploader = const _DisabledProfileImageUploader();

  final ProfileImagePickerPort _picker;
  final ProfileImageUploadPort _uploader;

  Future<SelectedProfileImage?> select() => _picker.pickFromGallery();

  Future<ProfileMetadata> resolve(
    ProfileMetadata metadata,
    SelectedProfileImage? selected,
  ) async {
    if (selected == null) return metadata;
    final pictureUrl = await _uploader.upload(selected);
    return metadata.withPicture(pictureUrl);
  }
}

final class _DisabledProfileImagePicker implements ProfileImagePickerPort {
  const _DisabledProfileImagePicker();

  @override
  Future<SelectedProfileImage?> pickFromGallery() async => null;
}

final class _DisabledProfileImageUploader implements ProfileImageUploadPort {
  const _DisabledProfileImageUploader();

  @override
  Future<ProfilePictureUrl> upload(SelectedProfileImage image) {
    throw StateError('Profile image upload is unavailable.');
  }
}
