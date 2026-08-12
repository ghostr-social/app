import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_nostr_account_generator.dart';

void main() {
  test(
    'pending setup storage failure returns safe retryable idle state',
    () async {
      final account = accountCreationAccount();
      final provisioning = _FailingRestoreProvisioning()
        ..restoreFailure = const AppFailure('Secure setup unavailable.');
      final cubit = AccountCreationCubit(
        FakeNostrAccountGenerator(account),
        provisioning,
        RecordingProfileRepository(),
      );

      await cubit.restorePending();

      expect(
        cubit.state,
        isA<AccountCreationIdle>().having(
          (state) => state.message,
          'message',
          'Secure setup unavailable.',
        ),
      );
      await cubit.close();
    },
  );
}

final class _FailingRestoreProvisioning
    extends FakeAccountProvisioningRepository {
  Object? restoreFailure;

  @override
  Future<RestoredPendingAccount?> restorePending() async =>
      throw restoreFailure!;
}
