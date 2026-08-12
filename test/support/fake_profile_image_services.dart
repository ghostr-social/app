import 'package:ghostr/features/profile/domain/profile_image_picker_port.dart';
import 'package:ghostr/features/profile/domain/profile_image_upload_port.dart';
import 'package:ghostr/features/profile/domain/profile_image_workflow.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';

SelectedProfileImage sampleProfileImage() => SelectedProfileImage(
  path: '/tmp/nora-avatar.png',
  label: 'nora-avatar.png',
  mimeType: ProfileImageMimeType.parse('image/png'),
);

final class FakeProfileImagePicker implements ProfileImagePickerPort {
  SelectedProfileImage? result;
  Object? failure;
  int pickCount = 0;

  @override
  Future<SelectedProfileImage?> pickFromGallery() async {
    pickCount += 1;
    if (failure case final error?) throw error;
    return result;
  }
}

final class FakeProfileImageUploader implements ProfileImageUploadPort {
  FakeProfileImageUploader({
    this.url = 'https://media.example/avatar.png',
    this.calls,
  });

  final String url;
  final List<String>? calls;
  Object? failure;
  SelectedProfileImage? uploaded;
  int uploadCount = 0;

  @override
  Future<ProfilePictureUrl> upload(SelectedProfileImage image) async {
    uploadCount += 1;
    calls?.add('uploadProfilePicture');
    uploaded = image;
    if (failure case final error?) throw error;
    return ProfilePictureUrl.parse(url);
  }
}

ProfileImageWorkflow fakeProfileImages({
  FakeProfileImagePicker? picker,
  FakeProfileImageUploader? uploader,
}) {
  return ProfileImageWorkflow(
    picker ?? FakeProfileImagePicker(),
    uploader ?? FakeProfileImageUploader(),
  );
}
