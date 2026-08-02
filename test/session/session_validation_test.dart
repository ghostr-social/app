import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/fakes.dart';

void main() {
  blocTest<SessionCubit, SessionState>(
    'shows a validation error for an invalid secret',
    build: () => SessionCubit(FakeSessionRepository()),
    act: (cubit) => cubit.signIn('npub1notvalid'),
    expect: () => [isA<SessionSignedOut>()],
    verify: (cubit) {
      expect((cubit.state as SessionSignedOut).errorMessage, isNotNull);
    },
  );
}
