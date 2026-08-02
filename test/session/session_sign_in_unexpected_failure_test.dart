import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/nostr_test_values.dart';

import '../support/fakes.dart';

void main() {
  blocTest<SessionCubit, SessionState>(
    'uses a safe message for an unexpected sign-in failure',
    build: () => SessionCubit(
      FakeSessionRepository(signInFailure: StateError('plugin failed')),
    ),
    act: (cubit) => cubit.signIn(testNsec),
    verify: (cubit) {
      expect(
        (cubit.state as SessionSignedOut).errorMessage,
        'Could not sign in securely.',
      );
    },
  );
}
