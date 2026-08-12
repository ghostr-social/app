import 'package:ghostr/features/session/domain/pending_account_setup.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

abstract interface class AccountProvisioningRepository {
  Future<void> stage(PendingAccountSetup setup);

  Future<RestoredPendingAccount?> restorePending();

  Future<UserSession> activate(PendingAccountSetup setup);

  Future<void> commit(PendingAccountSetup setup);

  Future<void> discard();
}
