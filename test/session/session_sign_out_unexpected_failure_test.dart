import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  blocTest<SessionCubit, SessionState>(
    'retains the session and clears an unexpected sign-out notice',
    build: () => SessionCubit(FakeSessionRepository(
      storedSession: sampleSession(),
      signOutFailure: StateError('plugin failed'),
    )),
    act: (cubit) async {
      await cubit.restore();
      await cubit.signOut();
      cubit.clearError();
    },
    expect: () => [
      isA<SessionLoading>(),
      isA<SessionSignedIn>(),
      isA<SessionSigningOut>(),
      isA<SessionSignedIn>().having(
        (state) => state.errorMessage,
        'message',
        'Could not sign out securely.',
      ),
      isA<SessionSignedIn>().having(
        (state) => state.errorMessage,
        'cleared message',
        isNull,
      ),
    ],
  );
}
