import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_account_generator.dart';

void main() {
  final account = accountCreationAccount();
  final metadata = accountCreationMetadata();
  final profiles = RecordingProfileRepository()
    ..saveFailure = const AppFailure('No relay accepted the profile.');

  blocTest<AccountCreationCubit, AccountCreationState>(
    'retains the generated account and exposes a safe publish failure',
    build: () => AccountCreationCubit(
      FakeNostrAccountGenerator(account),
      RecordingSessionRepository(account.identity),
      profiles,
    ),
    act: (cubit) async {
      await cubit.begin(metadata);
      await cubit.complete();
    },
    expect: () => [
      isA<AccountCreationStaging>(),
      isA<AccountCreationAwaitingBackup>(),
      isA<AccountCreationProvisioning>(),
      isA<AccountCreationFailure>()
          .having((state) => state.account, 'account', same(account))
          .having((state) => state.metadata, 'metadata', same(metadata))
          .having(
            (state) => state.account.secret,
            'secret',
            same(account.secret),
          )
          .having(
            (state) => state.message,
            'message',
            'No relay accepted the profile.',
          ),
    ],
  );
}
