import 'dart:async';

import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

final class PendingProfileMetadataRepository
    implements ProfileMetadataRepository {
  final pending = Completer<ProfileSummary?>();
  var refreshCount = 0;

  @override
  Future<ProfileSummary?> loadCached(ProfileId profileId) async => null;

  @override
  Future<ProfileSummary?> refresh(ProfileId profileId) {
    refreshCount += 1;
    return pending.future;
  }

  @override
  Future<ProfileSummary> save(
    NostrIdentity identity,
    ProfileMetadata metadata,
  ) {
    throw UnimplementedError();
  }
}
