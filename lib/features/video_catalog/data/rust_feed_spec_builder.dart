import 'dart:developer';

import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';
import 'package:ndk/ndk.dart';

/// Names the Rust feed one pull request opens, mirroring the request
/// precedence of `remoteVideoRetrievalContext`
/// (scheduled_remote_video_source.dart): search, then hashtag, then
/// profile, then the main feed — viewer-scoped when signed in, global
/// when not. Returns null when no feed can serve the request — ndk
/// parity: NdkVideoRemoteSource serves `const []` when a creator set
/// decodes to nothing.
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
    return FfiFeedSpec(kind: 'search', value: searchQuery, creators: const []);
  }
  if (hashtags == null || hashtags.isEmpty) return null;
  return FfiFeedSpec(
    kind: 'hashtag',
    value: hashtags.first,
    creators: const [],
  );
}

/// The main feed names the viewer only when one is signed in: Rust
/// degrades a viewer-less main feed to the unscoped global page ndk
/// serves signed out (discovery/feed_spec.rs, and
/// ndk_nostr_outbox_directory.dart knows no follows without an account).
/// A creator request names every creator it decoded — ndk queries the
/// whole set as `authors` (ndk_video_remote_source.dart), and the
/// Following feed asks for every follow at once.
FfiFeedSpec _identitySpec(List<String>? creators, String? viewerPubkeyHex) {
  if (creators != null) {
    return FfiFeedSpec(kind: 'profile', creators: creators);
  }
  return FfiFeedSpec(
    kind: 'main',
    creators: const [],
    viewerPubkey: viewerPubkeyHex,
  );
}

/// Creator ids decode exactly as the ndk source's `_decodeCreatorIds`
/// does (ndk_video_remote_source.dart): non-Nostr ids drop out.
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
