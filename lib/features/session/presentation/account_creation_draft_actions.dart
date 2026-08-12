part of 'account_creation_cubit.dart';

extension AccountCreationDraftActions on AccountCreationCubit {
  Future<void> restorePending() async {
    if (state is! AccountCreationIdle) return;
    emit(const AccountCreationRestoring());
    try {
      emit(_restoredState(await _provisioning.restorePending()));
    } on AppFailure catch (failure) {
      emit(AccountCreationIdle(message: failure.message));
    } on Object catch (error, stackTrace) {
      emit(AccountCreationIdle(message: _restoreFailure(error, stackTrace)));
    }
  }

  Future<void> begin(ProfileMetadata metadata) async {
    final idle = state;
    if (idle is! AccountCreationIdle) return;
    emit(AccountCreationStaging(selectedPicture: idle.selectedPicture));
    try {
      final setup = PendingAccountSetup(
        account: _generator.generate(),
        metadata: metadata,
        selectedPicture: idle.selectedPicture,
      );
      await _provisioning.stage(setup);
      emit(awaitingAccount(setup));
    } on AppFailure catch (failure) {
      _emitGenerationFailure(idle, failure.message);
    } on Object catch (error, stackTrace) {
      _emitGenerationFailure(idle, _generationFailure(error, stackTrace));
    }
  }

  Future<void> recoverProfile(ProfileMetadata metadata) async {
    final recovery = state;
    if (recovery is! AccountCreationProfileRecovery || recovery.isSubmitting) {
      return;
    }
    emit(AccountCreationProfileRecovery(recovery.account, isSubmitting: true));
    try {
      final setup = PendingAccountSetup(
        account: recovery.account,
        metadata: metadata,
      );
      await _provisioning.stage(setup);
      emit(awaitingAccount(setup));
    } on AppFailure catch (failure) {
      _rejectRecovery(recovery, failure.message);
    } on Object catch (error, stackTrace) {
      _rejectRecovery(recovery, _recoveryFailure(error, stackTrace));
    }
  }

  AccountCreationState _restoredState(RestoredPendingAccount? restored) {
    return switch (restored) {
      null => const AccountCreationIdle(),
      PendingAccountSetup() => awaitingAccount(restored),
      PendingAccountProfileRecovery(:final account) =>
        AccountCreationProfileRecovery(account),
    };
  }

  void _rejectRecovery(
    AccountCreationProfileRecovery recovery,
    String message,
  ) {
    emit(AccountCreationProfileRecovery(recovery.account, message: message));
  }

  String _restoreFailure(Object error, StackTrace stackTrace) {
    return _unexpected(
      'AccountCreationCubit.restorePending',
      'Could not restore unfinished account setup.',
      error,
      stackTrace,
    );
  }

  String _generationFailure(Object error, StackTrace stackTrace) {
    return _unexpected(
      'AccountCreationCubit.begin',
      'Could not generate a secure Nostr key.',
      error,
      stackTrace,
    );
  }

  String _recoveryFailure(Object error, StackTrace stackTrace) {
    return _unexpected(
      'AccountCreationCubit.recoverProfile',
      'Could not secure the recovered account profile.',
      error,
      stackTrace,
    );
  }
}
