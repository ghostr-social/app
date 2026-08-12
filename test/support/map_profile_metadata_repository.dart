import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

/// Serves cached profile summaries per profile id.
class MapProfileMetadataRepository implements ProfileMetadataRepository {
  MapProfileMetadataRepository([Map<ProfileId, ProfileSummary>? cached])
      : cached = {...?cached};

  final Map<ProfileId, ProfileSummary> cached;
  Object? loadFailure;

  @override
  Future<ProfileSummary?> loadCached(ProfileId profileId) async {
    if (loadFailure case final failure?) throw failure;
    return cached[profileId];
  }

  @override
  Future<ProfileSummary?> refresh(ProfileId profileId) async {
    return cached[profileId];
  }

  @override
  Future<ProfileSummary> save(
    NostrIdentity identity,
    ProfileMetadata metadata,
  ) async {
    final id = ProfileId.parse(identity.npub);
    return cached[id] = metadata.toSummary(id);
  }
}
