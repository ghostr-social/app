import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

final class ProfiledSessionRepository implements SessionRepository {
  const ProfiledSessionRepository(this._inner, this._profiles);

  final SessionRepository _inner;
  final ProfileMetadataRepository _profiles;

  @override
  Future<UserSession?> restore() async {
    final session = await _inner.restore();
    return session == null ? null : _withCachedProfile(session);
  }

  @override
  Future<UserSession> signIn(AuthSecret secret) async {
    return _withCachedProfile(await _inner.signIn(secret));
  }

  @override
  Future<void> signOut() => _inner.signOut();

  @override
  Future<void> resetStoredSession() => _inner.resetStoredSession();

  Future<UserSession> _withCachedProfile(UserSession session) async {
    try {
      final cached = await _profiles.loadCached(session.profile.id);
      return cached == null ? session : session.withProfile(cached);
    } on AppFailure {
      return session;
    }
  }
}
