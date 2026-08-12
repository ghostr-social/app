import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_nostr_account_generator.dart';

void main() {
  test('unexpected pending setup failure hides storage details', () async {
    final account = accountCreationAccount();
    final cubit = AccountCreationCubit(
      FakeNostrAccountGenerator(account),
      _UnexpectedRestoreProvisioning(),
      RecordingProfileRepository(),
    );

    await cubit.restorePending();

    expect(
      (cubit.state as AccountCreationIdle).message,
      'Could not restore unfinished account setup.',
    );
    await cubit.close();
  });
}

final class _UnexpectedRestoreProvisioning
    extends FakeAccountProvisioningRepository {
  @override
  Future<RestoredPendingAccount?> restorePending() async {
    throw StateError('private storage path');
  }
}
