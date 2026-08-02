import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/sample_data.dart';

void main() {
  blocTest<SessionCubit, SessionState>(
    'retains the signed-in session when sign out fails',
    build: () => SessionCubit(_SignOutFailureRepository()),
    act: (cubit) async {
      await cubit.restore();
      await cubit.signOut();
    },
    verify: (cubit) {
      final state = cubit.state as SessionSignedIn;
      expect(state.errorMessage, 'Could not sign out securely.');
      expect(state.session.identity.npub, sampleSession().identity.npub);
    },
  );
}

class _SignOutFailureRepository implements SessionRepository {
  @override
  Future<UserSession?> restore() async => sampleSession();

  @override
  Future<void> signOut() {
    throw const AppFailure('Could not sign out securely.');
  }

  @override
  Future<UserSession> signIn(AuthSecret secret) => throw UnimplementedError();

  @override
  Future<void> resetStoredSession() => throw UnimplementedError();
}
