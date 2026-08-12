import 'package:ghostr/features/session/domain/account_provisioning_repository.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

final class PendingFirstSessionRepository implements SessionRepository {
  const PendingFirstSessionRepository(this._inner, this._provisioning);

  final SessionRepository _inner;
  final AccountProvisioningRepository _provisioning;

  @override
  Future<UserSession?> restore() async {
    final pending = await _provisioning.restorePending();
    return pending == null ? _inner.restore() : null;
  }

  @override
  Future<UserSession> signIn(AuthSecret secret) async {
    await _provisioning.discard();
    return _inner.signIn(secret);
  }

  @override
  Future<void> signOut() => _inner.signOut();

  @override
  Future<void> resetStoredSession() async {
    await _provisioning.discard();
    await _inner.resetStoredSession();
  }
}
