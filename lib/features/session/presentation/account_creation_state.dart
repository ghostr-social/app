import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';
import 'package:ghostr/features/session/domain/generated_nostr_account.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

sealed class AccountCreationState {
  const AccountCreationState();
}

final class AccountCreationRestoring extends AccountCreationState {
  const AccountCreationRestoring();
}

final class AccountCreationStaging extends AccountCreationState {
  const AccountCreationStaging({this.selectedPicture});

  final SelectedProfileImage? selectedPicture;
}

final class AccountCreationIdle extends AccountCreationState {
  const AccountCreationIdle({this.selectedPicture, this.message});

  final SelectedProfileImage? selectedPicture;
  final String? message;
}

final class AccountCreationProfileRecovery extends AccountCreationState {
  const AccountCreationProfileRecovery(
    this.account, {
    this.isSubmitting = false,
    this.message,
  });

  final GeneratedNostrAccount account;
  final bool isSubmitting;
  final String? message;
}

final class AccountCreationSelectingPicture extends AccountCreationState {
  const AccountCreationSelectingPicture({this.selectedPicture});

  final SelectedProfileImage? selectedPicture;
}

final class AccountCreationAwaitingBackup extends AccountCreationState {
  const AccountCreationAwaitingBackup(
    this.account,
    this.metadata, {
    this.selectedPicture,
  });

  final GeneratedNostrAccount account;
  final ProfileMetadata metadata;
  final SelectedProfileImage? selectedPicture;
}

final class AccountCreationProvisioning extends AccountCreationState {
  const AccountCreationProvisioning(
    this.account,
    this.metadata, {
    this.selectedPicture,
  });

  final GeneratedNostrAccount account;
  final ProfileMetadata metadata;
  final SelectedProfileImage? selectedPicture;
}

final class AccountCreationFailure extends AccountCreationState {
  const AccountCreationFailure(
    this.account,
    this.metadata,
    this.message, {
    this.selectedPicture,
  });

  final GeneratedNostrAccount account;
  final ProfileMetadata metadata;
  final String message;
  final SelectedProfileImage? selectedPicture;
}

final class AccountCreationCompleted extends AccountCreationState {
  const AccountCreationCompleted(this.session);

  final UserSession session;
}
