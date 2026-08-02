import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/nostr_test_values.dart';

import '../support/fakes.dart';

void main() {
  blocTest<SessionCubit, SessionState>(
    'preserves an app-safe sign-in failure message',
    build: () => SessionCubit(FakeSessionRepository(
      signInFailure: const AppFailure('Relay signer unavailable.'),
    )),
    act: (cubit) => cubit.signIn(testNsec),
    verify: (cubit) {
      expect(
        (cubit.state as SessionSignedOut).errorMessage,
        'Relay signer unavailable.',
      );
    },
  );
}
