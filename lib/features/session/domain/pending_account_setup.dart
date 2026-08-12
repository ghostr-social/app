import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';
import 'package:ghostr/features/session/domain/generated_nostr_account.dart';

sealed class RestoredPendingAccount {
  const RestoredPendingAccount(this.account);

  final GeneratedNostrAccount account;
}

final class PendingAccountProfileRecovery extends RestoredPendingAccount {
  const PendingAccountProfileRecovery(super.account);
}

final class PendingAccountSetup extends RestoredPendingAccount {
  const PendingAccountSetup({
    required GeneratedNostrAccount account,
    required this.metadata,
    this.selectedPicture,
  }) : super(account);

  final ProfileMetadata metadata;
  final SelectedProfileImage? selectedPicture;
}
