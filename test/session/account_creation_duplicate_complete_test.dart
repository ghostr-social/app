import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_account_generator.dart';

void main() {
  final account = accountCreationAccount();
  final metadata = accountCreationMetadata();
  final gate = Completer<void>();
  final sessions = RecordingSessionRepository(
    account.identity,
    signInGate: gate,
  );
  final profiles = RecordingProfileRepository();

  blocTest<AccountCreationCubit, AccountCreationState>(
    'ignores duplicate completion while the first request is pending',
    build: () => AccountCreationCubit(
      FakeNostrAccountGenerator(account),
      sessions,
      profiles,
    ),
    act: (cubit) async {
      await cubit.begin(metadata);
      final first = cubit.complete();
      final duplicate = cubit.complete();
      gate.complete();
      await Future.wait([first, duplicate]);
    },
    expect: () => [
      isA<AccountCreationStaging>(),
      isA<AccountCreationAwaitingBackup>(),
      isA<AccountCreationProvisioning>(),
      isA<AccountCreationCompleted>(),
    ],
    verify: (_) {
      expect(sessions.signInSecrets, hasLength(1));
      expect(profiles.saveCount, 1);
    },
  );
}
