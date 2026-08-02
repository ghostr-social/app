import 'package:ghostr/features/session/domain/user_session.dart';

sealed class SessionState {
  const SessionState();
}

enum SessionOperation { awaitingRestore, restoring, signingIn, resetting }

class SessionLoading extends SessionState {
  const SessionLoading([
    this.operation = SessionOperation.awaitingRestore,
  ]);

  final SessionOperation operation;
}

class SessionSigningOut extends SessionState {
  const SessionSigningOut(this.session);

  final UserSession session;
}

class SessionSignedOut extends SessionState {
  const SessionSignedOut({this.errorMessage});

  final String? errorMessage;
}

class SessionRestoreFailure extends SessionState {
  const SessionRestoreFailure(this.message);

  final String message;
}

class SessionSignedIn extends SessionState {
  const SessionSignedIn(this.session, {this.errorMessage});

  final UserSession session;
  final String? errorMessage;
}
