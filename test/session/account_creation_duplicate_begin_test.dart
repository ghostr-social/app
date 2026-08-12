import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_nostr_account_generator.dart';

void main() {
  test('duplicate create taps generate and stage only one account', () async {
    final gate = Completer<void>();
    final account = accountCreationAccount();
    final generator = FakeNostrAccountGenerator(account);
    final provisioning = FakeAccountProvisioningRepository()..stageGate = gate;
    final cubit = AccountCreationCubit(
      generator,
      provisioning,
      RecordingProfileRepository(),
    );

    final first = cubit.begin(accountCreationMetadata());
    final duplicate = cubit.begin(accountCreationMetadata());
    expect(cubit.state, isA<AccountCreationStaging>());
    gate.complete();
    await Future.wait([first, duplicate]);

    expect(generator.generationCount, 1);
    expect(provisioning.stageCount, 1);
    await cubit.close();
  });
}
