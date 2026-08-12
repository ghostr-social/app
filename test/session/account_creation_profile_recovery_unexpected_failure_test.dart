import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_nostr_account_generator.dart';

void main() {
  test(
    'unexpected profile recovery failure is safe and retains the key',
    () async {
      final account = accountCreationAccount();
      final provisioning = FakeAccountProvisioningRepository()
        ..pending = PendingAccountProfileRecovery(account)
        ..stageFailure = StateError('plugin details');
      final cubit = AccountCreationCubit(
        FakeNostrAccountGenerator(account),
        provisioning,
        RecordingProfileRepository(),
      );

      await cubit.restorePending();
      await cubit.recoverProfile(accountCreationMetadata());

      final failed = cubit.state as AccountCreationProfileRecovery;
      expect(failed.account, same(account));
      expect(failed.message, 'Could not secure the recovered account profile.');
      expect(failed.message, isNot(contains('plugin details')));
      await cubit.close();
    },
  );
}
