import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/profile/domain/profile_image_picker_port.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';
import 'package:image_picker/image_picker.dart';

final class ImagePickerProfileImagePicker implements ProfileImagePickerPort {
  const ImagePickerProfileImagePicker(this._picker);

  final ImagePicker _picker;

  @override
  Future<SelectedProfileImage?> pickFromGallery() async {
    try {
      final file = await _picker.pickImage(
        source: ImageSource.gallery,
        maxWidth: 1024,
        maxHeight: 1024,
        imageQuality: 85,
      );
      return file == null ? null : _toImage(file);
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.profile.image_picker',
        message: 'Photo library access was denied or unavailable.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  SelectedProfileImage _toImage(XFile file) {
    final mimeType =
        ProfileImageMimeType.tryParse(file.mimeType) ??
        ProfileImageMimeType.fromFileName(file.name);
    return SelectedProfileImage(
      path: file.path,
      label: file.name,
      mimeType: mimeType,
    );
  }
}
