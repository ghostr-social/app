import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

extension type const ProfileDisplayName._(String value) implements String {}

extension type const ProfileHandle._(String value) implements String {}

extension type const ProfilePictureUrl._(String value) implements String {
  factory ProfilePictureUrl.parse(String raw) {
    final value = _picture(raw);
    if (value == null) {
      throw const FormatException('Picture must be an HTTP(S) URL.');
    }
    return value;
  }
}

final class ProfileMetadata {
  const ProfileMetadata._(this.displayName, this.handle, this.pictureUrl);

  factory ProfileMetadata.parse({
    required String displayName,
    required String handle,
    String? pictureUrl,
  }) {
    return ProfileMetadata._(
      ProfileDisplayName._(_displayName(displayName)),
      ProfileHandle._(_handle(handle)),
      _picture(pictureUrl),
    );
  }

  final ProfileDisplayName displayName;
  final ProfileHandle handle;
  final ProfilePictureUrl? pictureUrl;

  ProfileMetadata withPicture(ProfilePictureUrl? picture) {
    return ProfileMetadata._(displayName, handle, picture);
  }

  ProfileSummary toSummary(ProfileId id) {
    return ProfileSummary(
      id: id,
      displayName: displayName.value,
      handle: '@${handle.value}',
      avatarUrl: pictureUrl?.value,
    );
  }
}

String _displayName(String raw) {
  final value = raw.trim();
  if (value.isEmpty || value.length > 50) {
    throw const FormatException('Name must be between 1 and 50 characters.');
  }
  return value;
}

String _handle(String raw) {
  var value = raw.trim().toLowerCase();
  if (value.startsWith('@')) value = value.substring(1);
  if (!RegExp(r'^[a-z0-9_]{1,30}$').hasMatch(value)) {
    throw const FormatException(
      'Handle must use 1-30 letters, numbers, or underscores.',
    );
  }
  return value;
}

ProfilePictureUrl? _picture(String? raw) {
  final value = raw?.trim();
  if (value == null || value.isEmpty) return null;
  final uri = Uri.tryParse(value);
  if (uri == null ||
      !uri.hasAuthority ||
      uri.host.isEmpty ||
      uri.userInfo.isNotEmpty ||
      !_isWebScheme(uri.scheme)) {
    throw const FormatException('Picture must be an HTTP(S) URL.');
  }
  return ProfilePictureUrl._(uri.toString());
}

bool _isWebScheme(String scheme) => scheme == 'http' || scheme == 'https';
