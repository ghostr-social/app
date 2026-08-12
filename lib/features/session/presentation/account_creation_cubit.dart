import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/profile/domain/profile_image_workflow.dart';
import 'package:ghostr/features/session/domain/account_provisioning_repository.dart';
import 'package:ghostr/features/session/domain/nostr_account_generator.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';
import 'package:ghostr/features/session/presentation/account_creation_mappers.dart';
import 'package:ghostr/features/session/presentation/account_creation_state.dart';

export 'account_creation_state.dart';

part 'account_creation_draft_actions.dart';

final class AccountCreationCubit
    extends DisposalSafeCubit<AccountCreationState> {
  AccountCreationCubit(
    this._generator,
    this._provisioning,
    this._profiles, [
    this._images = const ProfileImageWorkflow.disabled(),
  ]) : super(const AccountCreationIdle());

  final NostrAccountGenerator _generator;
  final AccountProvisioningRepository _provisioning;
  final ProfileMetadataRepository _profiles;
  final ProfileImageWorkflow _images;

  Future<void> selectPicture() async {
    final idle = state;
    if (idle is! AccountCreationIdle) return;
    emit(
      AccountCreationSelectingPicture(selectedPicture: idle.selectedPicture),
    );
    try {
      final selected = await _images.select();
      emit(
        AccountCreationIdle(selectedPicture: selected ?? idle.selectedPicture),
      );
    } on AppFailure catch (failure) {
      emit(
        AccountCreationIdle(
          selectedPicture: idle.selectedPicture,
          message: failure.message,
        ),
      );
    } on Object catch (error, stackTrace) {
      final message = _unexpected(
        'AccountCreationCubit.selectPicture',
        'Could not select the profile picture.',
        error,
        stackTrace,
      );
      emit(
        AccountCreationIdle(
          selectedPicture: idle.selectedPicture,
          message: message,
        ),
      );
    }
  }

  Future<void> complete() async {
    final pending = _pending;
    if (pending == null) return;
    emit(
      AccountCreationProvisioning(
        pending.account,
        pending.metadata,
        selectedPicture: pending.selectedPicture,
      ),
    );
    try {
      final setup = pendingSetup(pending);
      final session = await _provisioning.activate(setup);
      final metadata = await _images.resolve(
        pending.metadata,
        pending.selectedPicture,
      );
      final profile = await _profiles.save(pending.account.identity, metadata);
      await _provisioning.commit(setup);
      emit(AccountCreationCompleted(session.withProfile(profile)));
    } on AppFailure catch (failure) {
      emit(accountCreationFailure(pending, failure.message));
    } on Object catch (error, stackTrace) {
      emit(
        accountCreationFailure(
          pending,
          _unexpected(
            'AccountCreationCubit.complete',
            'Could not finish creating this account.',
            error,
            stackTrace,
          ),
        ),
      );
    }
  }

  AccountCreationAwaitingBackup? get _pending {
    final current = state;
    return switch (current) {
      AccountCreationAwaitingBackup() => current,
      AccountCreationFailure() => AccountCreationAwaitingBackup(
        current.account,
        current.metadata,
        selectedPicture: current.selectedPicture,
      ),
      _ => null,
    };
  }

  void skipPicture() {
    final failure = state;
    if (failure is! AccountCreationFailure || failure.selectedPicture == null) {
      return;
    }
    emit(AccountCreationAwaitingBackup(failure.account, failure.metadata));
  }

  void reset() {
    if (state is AccountCreationCompleted) emit(const AccountCreationIdle());
  }

  void _emitGenerationFailure(AccountCreationIdle idle, String message) {
    emit(
      AccountCreationIdle(
        selectedPicture: idle.selectedPicture,
        message: message,
      ),
    );
  }

  String _unexpected(
    String source,
    String message,
    Object error,
    StackTrace stackTrace,
  ) {
    return translatedBoundaryFailure(
      source: source,
      message: message,
      error: error,
      stackTrace: stackTrace,
    ).message;
  }
}
