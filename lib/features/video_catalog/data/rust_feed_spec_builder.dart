import 'dart:developer';

import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';
import 'package:ndk/ndk.dart';

/// Names the Rust feed for one pull request.
///
/// Filter precedence is search, hashtag, profile, then main. The main
/// feed is viewer-scoped when signed in and global otherwise. A creator
/// request with no valid Nostr identities has no corresponding feed.
FfiFeedSpec? buildRustFeedSpec({
  Set<ProfileId>? creatorIds,
  String? searchQuery,
  Set<String>? hashtags,
  String? viewerPubkeyHex,
}) {
  final creators = creatorIds == null ? null : _decodedCreators(creatorIds);
  if (creators != null && creators.isEmpty) return null;
  return _termSpec(searchQuery, hashtags) ??
      _identitySpec(creators, viewerPubkeyHex);
}

FfiFeedSpec? _termSpec(String? searchQuery, Set<String>? hashtags) {
  if (searchQuery != null) {
    return FfiFeedSpec(
      kind: FfiFeedKind.search,
      value: searchQuery,
      creators: const [],
    );
  }
  if (hashtags == null || hashtags.isEmpty) return null;
  return FfiFeedSpec(
    kind: FfiFeedKind.hashtag,
    value: hashtags.first,
    creators: const [],
  );
}

/// Main feeds name the viewer only while signed in. Creator feeds name
/// every valid creator because the Following feed requests the set at once.
FfiFeedSpec _identitySpec(List<String>? creators, String? viewerPubkeyHex) {
  if (creators != null) {
    return FfiFeedSpec(kind: FfiFeedKind.profile, creators: creators);
  }
  return FfiFeedSpec(
    kind: FfiFeedKind.main,
    creators: const [],
    viewerPubkey: viewerPubkeyHex,
  );
}

/// Non-Nostr creator identifiers are ignored.
List<String> _decodedCreators(Set<ProfileId> creatorIds) {
  final decoded = <String>[];
  for (final creatorId in creatorIds) {
    try {
      decoded.add(NostrPublicKeyHex.parse(Nip19.decode(creatorId.value)).value);
    } on Object catch (error, stackTrace) {
      log(
        'Skipping a non-Nostr creator identifier.',
        name: 'ghostr.video.rustfeed',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
  return decoded;
}
