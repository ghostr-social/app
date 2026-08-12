import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/domain/generated_nostr_account.dart';
import 'package:ghostr/features/session/domain/nostr_account_generator.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';

void main() {
  test(
    'reports key generation failures and lets the profile form retry',
    () async {
      final account = accountCreationAccount();
      final generator = _FailingThenSuccessfulGenerator(account);
      final cubit = AccountCreationCubit(
        generator,
        RecordingSessionRepository(account.identity),
        RecordingProfileRepository(),
      );
      addTearDown(cubit.close);

      await cubit.begin(accountCreationMetadata());
      expect(
        cubit.state,
        isA<AccountCreationIdle>().having(
          (state) => state.message,
          'message',
          'Secure key generation unavailable.',
        ),
      );
      await cubit.begin(accountCreationMetadata());
      expect(cubit.state, isA<AccountCreationAwaitingBackup>());
      expect(generator.attempts, 2);
    },
  );
}

final class _FailingThenSuccessfulGenerator implements NostrAccountGenerator {
  _FailingThenSuccessfulGenerator(this.account);

  final GeneratedNostrAccount account;
  int attempts = 0;

  @override
  GeneratedNostrAccount generate() {
    attempts += 1;
    if (attempts == 1) {
      throw const AppFailure('Secure key generation unavailable.');
    }
    return account;
  }
}
