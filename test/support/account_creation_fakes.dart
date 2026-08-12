import 'dart:async';

import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/session/domain/account_provisioning_repository.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

export 'account_creation_values.dart';

final class RecordingSessionRepository
    implements SessionRepository, AccountProvisioningRepository {
  RecordingSessionRepository(this.identity, {this.calls, this.signInGate});

  final NostrIdentity identity;
  final List<String>? calls;
  final Completer<void>? signInGate;
  final List<AuthSecret> signInSecrets = [];
  PendingAccountSetup? pending;

  @override
  Future<void> stage(PendingAccountSetup setup) async => pending = setup;

  @override
  Future<RestoredPendingAccount?> restorePending() async => pending;

  @override
  Future<UserSession> activate(PendingAccountSetup setup) {
    return signIn(setup.account.secret);
  }

  @override
  Future<void> commit(PendingAccountSetup setup) async => pending = null;

  @override
  Future<void> discard() async => pending = null;

  @override
  Future<UserSession> signIn(AuthSecret secret) async {
    signInSecrets.add(secret);
    calls?.add('signIn');
    await signInGate?.future;
    return UserSession.fromIdentity(identity);
  }

  @override
  Future<UserSession?> restore() => throw UnimplementedError();

  @override
  Future<void> signOut() => throw UnimplementedError();

  @override
  Future<void> resetStoredSession() => throw UnimplementedError();
}

final class RecordingProfileRepository implements ProfileMetadataRepository {
  RecordingProfileRepository({this.calls});

  final List<String>? calls;
  Object? saveFailure;
  int saveCount = 0;
  NostrIdentity? savedIdentity;
  ProfileMetadata? savedMetadata;
  ProfileSummary? savedProfile;

  @override
  Future<ProfileSummary> save(
    NostrIdentity identity,
    ProfileMetadata metadata,
  ) async {
    saveCount += 1;
    calls?.add('saveProfile');
    if (saveFailure case final failure?) throw failure;
    savedIdentity = identity;
    savedMetadata = metadata;
    return savedProfile = metadata.toSummary(ProfileId.parse(identity.npub));
  }

  @override
  Future<ProfileSummary?> loadCached(ProfileId profileId) async => null;

  @override
  Future<ProfileSummary?> refresh(ProfileId profileId) async => null;
}
