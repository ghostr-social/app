import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

class UserSession {
  const UserSession._(this.secret, this.identity, this.profile);

  factory UserSession.fromIdentity(
    AuthSecret secret,
    NostrIdentity identity,
  ) {
    return UserSession._(
      secret,
      identity,
      ProfileSummary(
        id: ProfileId.parse(identity.npub),
        displayName: 'Nostr ${identity.npub.substring(5, 9).toUpperCase()}',
        handle: '@${identity.npub}',
        avatarUrl: null,
      ),
    );
  }

  final AuthSecret secret;
  final NostrIdentity identity;
  final ProfileSummary profile;
}
