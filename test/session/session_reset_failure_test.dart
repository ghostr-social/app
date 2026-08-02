import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/fakes.dart';

void main() {
  blocTest<SessionCubit, SessionState>(
    'preserves an app-safe stored-key reset failure',
    build: () => SessionCubit(FakeSessionRepository(
      resetFailure: const AppFailure('Secure key removal failed.'),
    )),
    act: (cubit) => cubit.resetStoredSession(),
    expect: () => [
      isA<SessionLoading>(),
      isA<SessionRestoreFailure>().having(
        (state) => state.message,
        'message',
        'Secure key removal failed.',
      ),
    ],
  );
}
