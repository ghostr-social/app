import 'dart:async';

import 'package:ghostr/features/session/domain/account_provisioning_repository.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

class FakeAccountProvisioningRepository
    implements AccountProvisioningRepository {
  RestoredPendingAccount? pending;
  Object? stageFailure;
  Object? discardFailure;
  Completer<void>? stageGate;
  int stageCount = 0;
  int discardCount = 0;
  int activateCount = 0;

  @override
  Future<void> stage(PendingAccountSetup setup) async {
    stageCount += 1;
    if (stageFailure case final failure?) throw failure;
    await stageGate?.future;
    pending = setup;
  }

  @override
  Future<RestoredPendingAccount?> restorePending() async => pending;

  @override
  Future<UserSession> activate(PendingAccountSetup setup) async {
    activateCount += 1;
    return UserSession.fromIdentity(setup.account.identity);
  }

  @override
  Future<void> commit(PendingAccountSetup setup) async => pending = null;

  @override
  Future<void> discard() async {
    discardCount += 1;
    if (discardFailure case final failure?) throw failure;
    pending = null;
  }
}
