import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/fakes.dart';

void main() {
  blocTest<SessionCubit, SessionState>(
    'translates an unexpected stored-key reset failure',
    build: () => SessionCubit(FakeSessionRepository(
      resetFailure: StateError('storage plugin failed'),
    )),
    act: (cubit) => cubit.resetStoredSession(),
    expect: () => [
      isA<SessionLoading>(),
      isA<SessionRestoreFailure>().having(
        (state) => state.message,
        'message',
        'Could not clear the stored key.',
      ),
    ],
  );
}
