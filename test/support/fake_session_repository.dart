import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

import 'nostr_test_values.dart';

class FakeSessionRepository implements SessionRepository {
  FakeSessionRepository({
    this.storedSession,
    this.restoreFailure,
    this.signInFailure,
    this.signOutFailure,
    this.resetFailure,
  });

  UserSession? storedSession;
  final Object? restoreFailure;
  final Object? signInFailure;
  final Object? signOutFailure;
  final Object? resetFailure;

  @override
  Future<UserSession?> restore() async {
    if (restoreFailure case final failure?) throw failure;
    return storedSession;
  }

  @override
  Future<void> signOut() async {
    if (signOutFailure case final failure?) throw failure;
    storedSession = null;
  }

  @override
  Future<void> resetStoredSession() async {
    if (resetFailure case final failure?) throw failure;
    storedSession = null;
  }

  @override
  Future<UserSession> signIn(AuthSecret secret) async {
    if (signInFailure case final failure?) throw failure;
    storedSession = UserSession.fromIdentity(
      NostrIdentity.parse(
        publicKeyHex: testViewerPublicKey,
        npub: testViewerNpub,
      ),
    );
    return storedSession!;
  }
}
