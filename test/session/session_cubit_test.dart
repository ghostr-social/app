import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  blocTest<SessionCubit, SessionState>(
    'restores a signed-in session when one exists',
    build: () =>
        SessionCubit(FakeSessionRepository(storedSession: sampleSession())),
    act: (cubit) => cubit.restore(),
    expect: () => [isA<SessionLoading>(), isA<SessionSignedIn>()],
  );
}
