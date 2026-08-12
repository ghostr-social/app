import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_account_generator.dart';

void main() {
  final account = accountCreationAccount();
  final metadata = accountCreationMetadata();
  final generator = FakeNostrAccountGenerator(account);
  final sessions = RecordingSessionRepository(account.identity);
  final profiles = RecordingProfileRepository()
    ..saveFailure = const AppFailure('Relay unavailable.');

  blocTest<AccountCreationCubit, AccountCreationState>(
    'retries completion with the original keypair',
    build: () => AccountCreationCubit(generator, sessions, profiles),
    act: (cubit) async {
      await cubit.begin(metadata);
      await cubit.complete();
      profiles.saveFailure = null;
      await cubit.complete();
    },
    expect: () => [
      isA<AccountCreationStaging>(),
      isA<AccountCreationAwaitingBackup>(),
      isA<AccountCreationProvisioning>(),
      isA<AccountCreationFailure>(),
      isA<AccountCreationProvisioning>(),
      isA<AccountCreationCompleted>(),
    ],
    verify: (_) {
      expect(generator.generationCount, 1);
      expect(sessions.signInSecrets, [
        same(account.secret),
        same(account.secret),
      ]);
      expect(profiles.saveCount, 2);
    },
  );
}
