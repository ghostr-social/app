import 'dart:convert';

import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_projection.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

final class NostrProfileMetadataMapper {
  const NostrProfileMetadataMapper();

  ProfileSummary summaryFromEvent(NostrEventRecord event, ProfileId id) {
    if (event.kind.value != 0) {
      throw const FormatException('Expected Nostr profile metadata.');
    }
    final payload = _object(event.content);
    final author = event.authorPublicKeyHex;
    final handle = projectRelayProfileIdentityHandle(payload['name'], author);
    return ProfileSummary(
      id: id,
      displayName:
          projectRelayProfileIdentityText(
            payload['display_name'],
            50,
            author,
          ) ??
          projectRelayProfileIdentityText(payload['name'], 50, author) ??
          _shortIdentity(id),
      handle: '@${handle ?? id.value}',
      avatarUrl: projectRelayProfilePicture(payload['picture']),
    );
  }

  NostrUnsignedEvent toEvent(
    ProfileMetadata metadata, {
    String? previousContent,
  }) {
    final payload = _previous(previousContent);
    payload['display_name'] = metadata.displayName.value;
    payload['name'] = metadata.handle.value;
    final picture = metadata.pictureUrl?.value;
    if (picture == null) {
      payload.remove('picture');
    } else {
      payload['picture'] = picture;
    }
    return NostrUnsignedEvent(
      kind: 0,
      tags: const [],
      content: jsonEncode(payload),
    );
  }

  Map<String, Object?> _previous(String? raw) {
    if (raw == null || raw.trim().isEmpty) return <String, Object?>{};
    try {
      return Map<String, Object?>.of(_object(raw));
    } on FormatException {
      return <String, Object?>{};
    }
  }

  Map<String, dynamic> _object(String raw) {
    final decoded = jsonDecode(raw);
    if (decoded is! Map<String, dynamic>) {
      throw const FormatException('Nostr profile metadata must be an object.');
    }
    return decoded;
  }

  String _shortIdentity(ProfileId id) {
    final runes = id.value.runes;
    return runes.length <= 12
        ? id.value
        : '${String.fromCharCodes(runes.take(12))}…';
  }
}
