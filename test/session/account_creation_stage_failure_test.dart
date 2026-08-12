import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_nostr_account_generator.dart';

void main() {
  test(
    'stage failure never exposes the generated secret and can retry',
    () async {
      final account = accountCreationAccount();
      final generator = FakeNostrAccountGenerator(account);
      final provisioning = FakeAccountProvisioningRepository()
        ..stageFailure = const AppFailure('Secure setup unavailable.');
      final cubit = AccountCreationCubit(
        generator,
        provisioning,
        RecordingProfileRepository(),
      );

      await cubit.begin(accountCreationMetadata());
      expect(
        cubit.state,
        isA<AccountCreationIdle>().having(
          (state) => state.message,
          'message',
          'Secure setup unavailable.',
        ),
      );
      provisioning.stageFailure = null;
      await cubit.begin(accountCreationMetadata());

      expect(cubit.state, isA<AccountCreationAwaitingBackup>());
      expect(generator.generationCount, 2);
      await cubit.close();
    },
  );
}
