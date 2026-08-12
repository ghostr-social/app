import 'package:ghostr/features/session/domain/pending_account_setup.dart';
import 'package:ghostr/features/session/presentation/account_creation_state.dart';

AccountCreationAwaitingBackup awaitingAccount(PendingAccountSetup setup) {
  return AccountCreationAwaitingBackup(
    setup.account,
    setup.metadata,
    selectedPicture: setup.selectedPicture,
  );
}

PendingAccountSetup pendingSetup(AccountCreationAwaitingBackup pending) {
  return PendingAccountSetup(
    account: pending.account,
    metadata: pending.metadata,
    selectedPicture: pending.selectedPicture,
  );
}

AccountCreationFailure accountCreationFailure(
  AccountCreationAwaitingBackup pending,
  String message,
) {
  return AccountCreationFailure(
    pending.account,
    pending.metadata,
    message,
    selectedPicture: pending.selectedPicture,
  );
}
