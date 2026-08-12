import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

class FakeProfileMetadataRepository implements ProfileMetadataRepository {
  ProfileSummary? cached;
  Object? loadFailure;
  Object? refreshFailure;
  Object? saveFailure;
  ProfileMetadata? savedMetadata;
  NostrIdentity? savedIdentity;

  @override
  Future<ProfileSummary?> loadCached(ProfileId profileId) async {
    if (loadFailure case final failure?) throw failure;
    return cached;
  }

  @override
  Future<ProfileSummary?> refresh(ProfileId profileId) async {
    if (refreshFailure case final failure?) throw failure;
    return cached;
  }

  @override
  Future<ProfileSummary> save(
    NostrIdentity identity,
    ProfileMetadata metadata,
  ) async {
    if (saveFailure case final failure?) throw failure;
    savedIdentity = identity;
    savedMetadata = metadata;
    return cached = metadata.toSummary(ProfileId.parse(identity.npub));
  }
}
