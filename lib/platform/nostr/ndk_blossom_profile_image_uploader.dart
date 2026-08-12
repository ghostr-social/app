import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/profile/domain/profile_image_upload_port.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ndk/ndk.dart';

final class NdkBlossomProfileImageUploader implements ProfileImageUploadPort {
  NdkBlossomProfileImageUploader({
    required Ndk ndk,
    required List<BlossomServerUrl> servers,
  }) : _ndk = ndk,
       _servers = servers.map((server) => server.value).toList();

  final Ndk _ndk;
  final List<String> _servers;

  @override
  Future<ProfilePictureUrl> upload(SelectedProfileImage image) async {
    try {
      final progress = await _ndk.files
          .uploadFromFile(
            filePath: image.path,
            serverUrls: _servers,
            contentType: image.mimeType.value,
          )
          .last;
      return _pictureFrom(progress, image.mimeType);
    } on Object catch (error, stackTrace) {
      if (error is AppFailure) rethrow;
      throw translatedBoundaryFailure(
        source: 'ghostr.profile.blossom',
        message: 'Could not upload the profile picture.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  ProfilePictureUrl _pictureFrom(
    BlobUploadProgress progress,
    ProfileImageMimeType expectedMimeType,
  ) {
    if (!progress.isComplete) {
      throw const AppFailure('No Blossom server accepted the picture.');
    }
    for (final result in progress.completedUploads) {
      final descriptor = result.descriptor;
      if (result.success && _matches(descriptor, expectedMimeType)) {
        return ProfilePictureUrl.parse(descriptor!.url);
      }
    }
    throw const AppFailure('No Blossom server accepted the picture.');
  }

  bool _matches(
    BlobDescriptor? descriptor,
    ProfileImageMimeType expectedMimeType,
  ) {
    if (descriptor == null || (descriptor.size ?? 0) <= 0) return false;
    if (!_isHttpUrl(descriptor.url)) return false;
    final mimeType = descriptor.type?.trim().toLowerCase();
    return mimeType == null ||
        mimeType.isEmpty ||
        mimeType == expectedMimeType.value;
  }

  bool _isHttpUrl(String raw) {
    final uri = Uri.tryParse(raw);
    if (uri == null || uri.host.isEmpty) return false;
    return uri.scheme == 'https' || uri.scheme == 'http';
  }
}
