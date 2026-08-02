import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/fakes.dart';

void main() {
  blocTest<SessionCubit, SessionState>(
    'uses a safe message for an unexpected restore failure',
    build: () => SessionCubit(
      FakeSessionRepository(restoreFailure: StateError('plugin failed')),
    ),
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
