import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_nostr_account_generator.dart';

void main() {
  test('profile recovery failure preserves the staged key for retry', () async {
    final account = accountCreationAccount();
    final generator = FakeNostrAccountGenerator(account);
    final provisioning = FakeAccountProvisioningRepository()
      ..pending = PendingAccountProfileRecovery(account)
      ..stageFailure = const AppFailure('Secure setup unavailable.');
    final cubit = AccountCreationCubit(
      generator,
      provisioning,
      RecordingProfileRepository(),
    );

    await cubit.restorePending();
    await cubit.recoverProfile(accountCreationMetadata());

    final failed = cubit.state as AccountCreationProfileRecovery;
    expect(failed.account, same(account));
    expect(failed.message, 'Secure setup unavailable.');
    expect(generator.generationCount, 0);
    provisioning.stageFailure = null;
    await cubit.recoverProfile(accountCreationMetadata());
    expect(cubit.state, isA<AccountCreationAwaitingBackup>());
    await cubit.close();
  });
}
