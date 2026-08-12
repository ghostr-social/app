import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';

abstract interface class ProfileImageUploadPort {
  Future<ProfilePictureUrl> upload(SelectedProfileImage image);
}
