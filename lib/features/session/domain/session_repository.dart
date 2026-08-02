import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

abstract interface class SessionRepository {
  Future<UserSession?> restore();

  Future<UserSession> signIn(AuthSecret secret);

  Future<void> signOut();

  Future<void> resetStoredSession();
}
