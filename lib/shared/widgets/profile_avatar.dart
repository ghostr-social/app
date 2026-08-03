import 'package:flutter/material.dart';

class ProfileAvatar extends StatelessWidget {
  const ProfileAvatar({
    required this.initials,
    required this.avatarUrl,
    this.radius,
    super.key,
  });

  final String initials;
  final String? avatarUrl;
  final double? radius;

  @override
  Widget build(BuildContext context) {
    final image = remoteAvatarImage(avatarUrl);
    return CircleAvatar(
      radius: radius,
      foregroundImage: image,
      onForegroundImageError: image == null ? null : (_, __) {},
      child: Text(initials),
    );
  }
}

ImageProvider? remoteAvatarImage(String? url) {
  final trimmed = url?.trim();
  if (trimmed == null || trimmed.isEmpty) return null;
  final parsed = Uri.tryParse(trimmed);
  if (parsed == null || parsed.host.isEmpty) return null;
  if (parsed.scheme != 'http' && parsed.scheme != 'https') return null;
  return NetworkImage(parsed.toString());
}
