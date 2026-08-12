import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_nostr_account_generator.dart';

void main() {
  test(
    'picture skip is ignored unless a failed selected picture exists',
    () async {
      final account = accountCreationAccount();
      final profiles = RecordingProfileRepository()
        ..saveFailure = const AppFailure('Relay unavailable.');
      final cubit = AccountCreationCubit(
        FakeNostrAccountGenerator(account),
        FakeAccountProvisioningRepository(),
        profiles,
      );

      final idle = cubit.state;
      cubit.skipPicture();
      expect(cubit.state, same(idle));

      await cubit.begin(accountCreationMetadata());
      await cubit.complete();
      final failure = cubit.state;
      cubit.skipPicture();
      expect(cubit.state, same(failure));
      await cubit.close();
    },
  );
}
