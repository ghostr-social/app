import 'package:ghostr/core/storage/secret_store.dart';
import 'package:ghostr/features/session/domain/nostr_session_port.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity_deriver.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

class SecureSessionRepository implements SessionRepository {
  SecureSessionRepository(
    this._secretStore,
    this._identityDeriver,
    this._nostrSession,
  );

  final SecretStore _secretStore;
  final NostrIdentityDeriver _identityDeriver;
  final NostrSessionPort _nostrSession;

  // Activation failures leave the stored secret alone: the engine being
  // unavailable is retryable, and clearing would silently discard the
  // viewer's identity along with everything namespaced under it. The
  // session gate offers "Use another key" for a deliberate reset.
  @override
  Future<UserSession?> restore() async {
    final stored = await _secretStore.read();
    final secret = AuthSecret.tryParse(stored ?? '');
    if (secret == null) {
      return null;
    }
    return _activate(secret);
  }

  @override
  Future<UserSession> signIn(AuthSecret secret) async {
    final identity = _identityDeriver.derive(secret);
    final session = UserSession.fromIdentity(identity);
    await _secretStore.write(secret.value);
    try {
      await _nostrSession.activate(secret, identity);
    } on Object {
      await _secretStore.clear();
      rethrow;
    }
    return session;
  }

  @override
  Future<void> signOut() async {
    final stored = await _secretStore.read();
    await _secretStore.clear();
    try {
      await _nostrSession.deactivate();
    } on Object {
      if (stored != null) await _secretStore.write(stored);
      rethrow;
    }
  }

  @override
  Future<void> resetStoredSession() async {
    await _secretStore.clear();
    await _nostrSession.deactivate();
  }

  Future<UserSession> _activate(AuthSecret secret) async {
    final identity = _identityDeriver.derive(secret);
    await _nostrSession.activate(secret, identity);
    return UserSession.fromIdentity(identity);
  }
}
