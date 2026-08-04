import 'dart:developer';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';
import 'package:ndk/ndk.dart';

/// Names the Rust feed one pull request opens, mirroring the request
/// precedence of `remoteVideoRetrievalContext`
/// (scheduled_remote_video_source.dart): search, then hashtag, then
/// profile, then the viewer's main feed. Returns null when no feed can
/// serve the request — ndk parity: NdkVideoRemoteSource serves
/// `const []` when a creator set decodes to nothing.
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
    return FfiFeedSpec(kind: 'search', value: searchQuery);
  }
  if (hashtags == null || hashtags.isEmpty) return null;
  return FfiFeedSpec(kind: 'hashtag', value: hashtags.first);
}

FfiFeedSpec _identitySpec(List<String>? creators, String? viewerPubkeyHex) {
  if (creators != null) {
    return FfiFeedSpec(kind: 'profile', value: creators.first);
  }
  if (viewerPubkeyHex == null) {
    // The Rust main feed is viewer-scoped (api::feed_types FfiFeedSpec).
    throw const AppFailure('Sign in to load the Rust main feed.');
  }
  return FfiFeedSpec(kind: 'main', viewerPubkey: viewerPubkeyHex);
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
