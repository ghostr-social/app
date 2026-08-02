import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

class ProfileSummary {
  const ProfileSummary({
    required this.id,
    required this.displayName,
    required this.handle,
    required this.avatarUrl,
  });

  factory ProfileSummary.unknown(ProfileId id) {
    return ProfileSummary(
      id: id,
      displayName: 'Unknown creator',
      handle: '@ghostr',
      avatarUrl: null,
    );
  }

  final ProfileId id;
  final String displayName;
  final String handle;
  final String? avatarUrl;

  String get initials {
    final words =
        displayName.trim().split(' ').where((word) => word.isNotEmpty);
    final pairs = words.take(2).map((word) => word.substring(0, 1));
    return pairs.join().toUpperCase();
  }
}
