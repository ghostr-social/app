import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

void main() {
  blocTest<SessionCubit, SessionState>(
    'models a retryable session-restore failure',
    build: () => SessionCubit(_RestoreFailureRepository()),
    act: (cubit) => cubit.restore(),
    expect: () => [
      isA<SessionLoading>(),
      isA<SessionRestoreFailure>().having(
        (state) => state.message,
        'message',
        'Secure session unavailable.',
      ),
    ],
  );
}

class _RestoreFailureRepository implements SessionRepository {
  @override
  Future<UserSession?> restore() {
    throw const AppFailure('Secure session unavailable.');
  }

  @override
  Future<UserSession> signIn(AuthSecret secret) => throw UnimplementedError();

  @override
  Future<void> signOut() => throw UnimplementedError();

  @override
  Future<void> resetStoredSession() => throw UnimplementedError();
}
