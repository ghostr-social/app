import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/fakes.dart';
import '../support/nostr_test_values.dart';

void main() {
  blocTest<SessionCubit, SessionState>(
    'translates a repository FormatException into safe validation feedback',
    build: () => SessionCubit(FakeSessionRepository(
      signInFailure: const FormatException('invalid stored key'),
    )),
    act: (cubit) => cubit.signIn(testNsec),
    expect: () => [
      isA<SessionLoading>(),
      isA<SessionSignedOut>().having(
        (state) => state.errorMessage,
        'errorMessage',
        'Enter a valid nsec1 secret.',
      ),
    ],
  );
}
