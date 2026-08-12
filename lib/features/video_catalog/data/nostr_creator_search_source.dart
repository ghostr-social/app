import 'dart:convert';

import 'package:ghostr/core/nostr/nostr_bech32.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_projection.dart';
import 'package:ghostr/features/video_catalog/domain/creator_search_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

final class NostrCreatorSearchSource implements CreatorSearchSource {
  const NostrCreatorSearchSource(this._client);

  final NostrEventClient _client;

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async {
    final term = query.trim();
    if (term.isEmpty) return const <ProfileSummary>[];
    final events = await _client.query(_profileQuery(term));
    return _newestProfiles(events);
  }

  NostrEventQuery _profileQuery(String term) {
    final author = _directAuthor(term);
    return NostrEventQuery(
      kinds: const <int>[0],
      scope: author == null ? null : NostrEventQueryScope(authors: [author]),
      limit: author == null ? 30 : 1,
      search: author == null ? term : null,
    );
  }

  NostrPublicKeyHex? _directAuthor(String term) {
    final bytes = term.startsWith('npub1')
        ? decodeNostrBech32Key(term, 'npub')
        : nostrKeyBytes(term);
    if (bytes == null) return null;
    return NostrPublicKeyHex.parse(nostrKeyHex(bytes));
  }

  List<ProfileSummary> _newestProfiles(List<NostrEventRecord> events) {
    final newest = <NostrPublicKeyHex, _MetadataResult>{};
    for (final event in events) {
      final metadata = _metadata(event);
      if (metadata == null) continue;
      final current = newest[event.authorPublicKeyHex];
      if (current == null || _isNewer(event, current.event)) {
        newest[event.authorPublicKeyHex] = (event: event, metadata: metadata);
      }
    }
    return newest.values.map(_profile).toList(growable: false);
  }

  bool _isNewer(NostrEventRecord candidate, NostrEventRecord current) {
    if (candidate.createdAt != current.createdAt) {
      return candidate.createdAt > current.createdAt;
    }
    return candidate.id.compareTo(current.id) < 0;
  }

  _ProfileMetadata? _metadata(NostrEventRecord event) {
    if (event.kind.value != 0) return null;
    try {
      final payload = jsonDecode(event.content);
      if (payload is! Map<String, dynamic>) return null;
      final displayName = _text(payload, 'display_name');
      final name = _text(payload, 'name');
      final picture = _text(payload, 'picture');
      return (
        displayName: projectRelayProfileIdentityText(
          displayName,
          50,
          event.authorPublicKeyHex,
        ),
        name: projectRelayProfileIdentityText(
          name,
          50,
          event.authorPublicKeyHex,
        ),
        handle: projectRelayProfileIdentityHandle(
          name,
          event.authorPublicKeyHex,
        ),
        picture: projectRelayProfilePicture(picture),
      );
    } on FormatException {
      return null;
    }
  }

  String? _text(Map<String, dynamic> payload, String key) {
    final value = payload[key];
    if (value == null || value is String) return value as String?;
    throw FormatException('Nostr profile field "$key" must be text.');
  }

  ProfileSummary _profile(_MetadataResult result) {
    final publicKey = result.event.authorPublicKeyHex.value;
    final npub = encodeNostrBech32Key('npub', nostrKeyBytes(publicKey)!)!;
    return ProfileSummary(
      id: ProfileId.parse(npub),
      displayName: _profileName(result.metadata, npub),
      handle: _profileHandle(result.metadata, npub),
      avatarUrl: result.metadata.picture,
    );
  }

  String _profileName(_ProfileMetadata metadata, String npub) {
    for (final candidate in [metadata.displayName, metadata.name]) {
      if (candidate != null && candidate.trim().isNotEmpty) return candidate;
    }
    return '${npub.substring(0, 12)}…';
  }

  String _profileHandle(_ProfileMetadata metadata, String npub) {
    return '@${metadata.handle ?? npub}';
  }
}

typedef _ProfileMetadata = ({
  String? displayName,
  String? name,
  String? handle,
  String? picture,
});

typedef _MetadataResult = ({NostrEventRecord event, _ProfileMetadata metadata});
