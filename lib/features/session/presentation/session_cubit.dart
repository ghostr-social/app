import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/session/presentation/session_state.dart';

export 'session_state.dart';

class SessionCubit extends DisposalSafeCubit<SessionState> {
  SessionCubit(this._repository) : super(const SessionLoading());

  final SessionRepository _repository;

  Future<void> restore() async {
    if (!_start(const SessionLoading(SessionOperation.restoring))) return;
    try {
      _acceptRestored(await _repository.restore());
    } on AppFailure catch (failure) {
      emit(SessionRestoreFailure(failure.message));
    } on Object catch (error, stackTrace) {
      emit(SessionRestoreFailure(_restoreError(error, stackTrace)));
    }
  }

  Future<void> signIn(String rawSecret) async {
    if (!_canStart) return;
    final secret = AuthSecret.tryParse(rawSecret);
    if (secret == null) {
      _emitInvalidSecret();
      return;
    }
    emit(const SessionLoading(SessionOperation.signingIn));
    await _completeSignIn(secret);
  }

  Future<void> _completeSignIn(AuthSecret secret) async {
    try {
      emit(SessionSignedIn(await _repository.signIn(secret)));
    } on FormatException {
      _emitInvalidSecret();
    } on AppFailure catch (failure) {
      emit(SessionSignedOut(errorMessage: failure.message));
    } on Object catch (error, stackTrace) {
      emit(SessionSignedOut(errorMessage: _signInError(error, stackTrace)));
    }
  }

  Future<void> signOut() async {
    final signedIn = state;
    if (signedIn is! SessionSignedIn ||
        !_start(SessionSigningOut(signedIn.session))) {
      return;
    }
    try {
      await _repository.signOut();
      emit(const SessionSignedOut());
    } on AppFailure catch (failure) {
      emit(SessionSignedIn(signedIn.session, errorMessage: failure.message));
    } on Object catch (error, stackTrace) {
      emit(SessionSignedIn(
        signedIn.session,
        errorMessage: _signOutError(error, stackTrace),
      ));
    }
  }

  Future<void> resetStoredSession() async {
    if (!_start(const SessionLoading(SessionOperation.resetting))) return;
    try {
      await _repository.resetStoredSession();
      emit(const SessionSignedOut());
    } on AppFailure catch (failure) {
      emit(SessionRestoreFailure(failure.message));
    } on Object catch (error, stackTrace) {
      emit(SessionRestoreFailure(_resetError(error, stackTrace)));
    }
  }

  void clearError() {
    final signedIn = state;
    if (signedIn is SessionSignedIn && signedIn.errorMessage != null) {
      emit(SessionSignedIn(signedIn.session));
    }
  }

  bool _start(SessionState pending) {
    if (!_canStart) return false;
    emit(pending);
    return true;
  }

  bool get _canStart {
    final current = state;
    if (isClosed || current is SessionSigningOut) return false;
    return current is! SessionLoading ||
        current.operation == SessionOperation.awaitingRestore;
  }

  void _acceptRestored(UserSession? session) {
    emit(session == null ? const SessionSignedOut() : SessionSignedIn(session));
  }

  void _emitInvalidSecret() {
    emit(const SessionSignedOut(errorMessage: 'Enter a valid nsec1 secret.'));
  }

  String _restoreError(Object error, StackTrace stackTrace) {
    return _unexpected(
      'SessionCubit.restore',
      'Secure session unavailable.',
      error,
      stackTrace,
    );
  }

  String _signInError(Object error, StackTrace stackTrace) {
    return _unexpected(
      'SessionCubit.signIn',
      'Could not sign in securely.',
      error,
      stackTrace,
    );
  }

  String _signOutError(Object error, StackTrace stackTrace) {
    return _unexpected(
      'SessionCubit.signOut',
      'Could not sign out securely.',
      error,
      stackTrace,
    );
  }

  String _resetError(Object error, StackTrace stackTrace) {
    return _unexpected(
      'SessionCubit.resetStoredSession',
      'Could not clear the stored key.',
      error,
      stackTrace,
    );
  }

  String _unexpected(
    String source,
    String message,
    Object error,
    StackTrace stackTrace,
  ) {
    return translatedBoundaryFailure(
      source: source,
      message: message,
      error: error,
      stackTrace: stackTrace,
    ).message;
  }
}
