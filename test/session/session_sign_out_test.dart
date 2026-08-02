import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/fakes.dart';
import '../support/nostr_test_values.dart';

void main() {
  blocTest<SessionCubit, SessionState>(
    'signs in with a valid secret and signs out again',
    build: () => SessionCubit(FakeSessionRepository()),
    act: (cubit) async {
      await cubit.signIn(testNsec);
      await cubit.signOut();
    },
    expect: () => [
      isA<SessionLoading>(),
      isA<SessionSignedIn>(),
      isA<SessionSigningOut>(),
      isA<SessionSignedOut>(),
    ],
  );
}
