import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_account_generator.dart';

void main() {
  final account = accountCreationAccount();
  final metadata = accountCreationMetadata();
  final generator = FakeNostrAccountGenerator(account);

  blocTest<AccountCreationCubit, AccountCreationState>(
    'generates one account and exposes its secret for backup',
    build: () => AccountCreationCubit(
      generator,
      RecordingSessionRepository(account.identity),
      RecordingProfileRepository(),
    ),
    act: (cubit) => cubit.begin(metadata),
    expect: () => [
      isA<AccountCreationStaging>(),
      isA<AccountCreationAwaitingBackup>()
          .having((state) => state.account, 'account', same(account))
          .having((state) => state.metadata, 'metadata', same(metadata))
          .having(
            (state) => state.account.secret,
            'secret',
            same(account.secret),
          ),
    ],
    verify: (_) => expect(generator.generationCount, 1),
  );
}
