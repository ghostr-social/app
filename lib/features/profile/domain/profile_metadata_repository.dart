import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

abstract interface class ProfileMetadataRepository {
  Future<ProfileSummary?> loadCached(ProfileId profileId);

  Future<ProfileSummary?> refresh(ProfileId profileId);

  Future<ProfileSummary> save(NostrIdentity identity, ProfileMetadata metadata);
}
