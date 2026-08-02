import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/ndk_nostr_identity_deriver.dart';
import 'package:ghostr/features/session/data/secure_session_repository.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/memory_secret_store.dart';
import '../support/fake_nostr_session_port.dart';

void main() {
  blocTest<SessionCubit, SessionState>(
    'returns to sign in when an nsec has an invalid checksum',
    build: () => SessionCubit(
      SecureSessionRepository(
        MemorySecretStore(),
        const NdkNostrIdentityDeriver(),
        FakeNostrSessionPort(),
      ),
    ),
    act: (cubit) => cubit.signIn('nsec1validghostrsecretvalue123456'),
    expect: () => [
      isA<SessionSignedOut>().having(
        (state) => state.errorMessage,
        'error message',
        'Enter a valid nsec1 secret.',
      ),
    ],
  );
}
