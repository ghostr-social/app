import 'package:ghostr/features/profile/domain/selected_profile_image.dart';

abstract interface class ProfileImagePickerPort {
  Future<SelectedProfileImage?> pickFromGallery();
}
