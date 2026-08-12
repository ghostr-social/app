import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

class UserSession {
  const UserSession._(this.identity, this.profile);

  factory UserSession.fromIdentity(NostrIdentity identity) {
    return UserSession._(
      identity,
      ProfileSummary(
        id: ProfileId.parse(identity.npub),
        displayName: 'Nostr ${identity.npub.substring(5, 9).toUpperCase()}',
        handle: '@${identity.npub}',
        avatarUrl: null,
      ),
    );
  }

  final NostrIdentity identity;
  final ProfileSummary profile;

  UserSession withProfile(ProfileSummary updated) {
    if (updated.id != profile.id) {
      throw StateError('Session profile must match the Nostr identity.');
    }
    return UserSession._(identity, updated);
  }
}
