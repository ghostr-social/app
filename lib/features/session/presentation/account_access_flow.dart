import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';
import 'package:ghostr/features/session/presentation/create_account_profile_screen.dart';
import 'package:ghostr/features/session/presentation/existing_key_screen.dart';
import 'package:ghostr/features/session/presentation/onboarding_welcome_screen.dart';
import 'package:ghostr/features/session/presentation/private_key_backup_screen.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/secret_backup_port.dart';
import 'package:ghostr/features/profile/domain/selected_profile_image.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';

enum _AccessPage { welcome, existingKey, profile }

final class AccountAccessFlow extends StatefulWidget {
  const AccountAccessFlow({
    required this.secretBackup,
    this.errorMessage,
    this.isSigningIn = false,
    super.key,
  });

  final SecretBackupPort secretBackup;
  final String? errorMessage;
  final bool isSigningIn;

  @override
  State<AccountAccessFlow> createState() => _AccountAccessFlowState();
}

final class _AccountAccessFlowState extends State<AccountAccessFlow> {
  _AccessPage _page = _AccessPage.welcome;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) context.read<AccountCreationCubit>().reset();
    });
  }

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<AccountCreationCubit, AccountCreationState>(
      listenWhen: (_, state) => state is AccountCreationCompleted,
      listener: _acceptCreated,
      builder: _content,
    );
  }

  Widget _content(BuildContext context, AccountCreationState creation) {
    if (creation is AccountCreationRestoring) return _restoring();
    if (creation case AccountCreationAwaitingBackup(:final account)) {
      return _backup(context, account.secret, false, null);
    }
    if (creation case AccountCreationProvisioning(:final account)) {
      return _backup(context, account.secret, true, null);
    }
    if (creation case AccountCreationFailure(
      :final account,
      :final message,
      :final selectedPicture,
    )) {
      return _backup(
        context,
        account.secret,
        false,
        message,
        canSkipPicture: selectedPicture != null,
      );
    }
    if (creation case AccountCreationProfileRecovery()) {
      return _recoveredProfile(context, creation);
    }
    return switch (_page) {
      _AccessPage.welcome => OnboardingWelcomeScreen(
        onCreateAccount: () => _show(_AccessPage.profile),
        onUseExistingKey: () => _show(_AccessPage.existingKey),
      ),
      _AccessPage.existingKey => ExistingKeyScreen(
        errorMessage: widget.errorMessage,
        isSigningIn: widget.isSigningIn,
        onSubmit: context.read<SessionCubit>().signIn,
        onBack: () => _show(_AccessPage.welcome),
      ),
      _AccessPage.profile => CreateAccountProfileScreen(
        initial: null,
        onSubmit: context.read<AccountCreationCubit>().begin,
        selectedPicture: _selectedPicture(creation),
        isSelectingPicture: creation is AccountCreationSelectingPicture,
        isSubmitting: creation is AccountCreationStaging,
        onSelectPicture: context.read<AccountCreationCubit>().selectPicture,
        errorMessage: creation is AccountCreationIdle ? creation.message : null,
        onBack: () => _show(_AccessPage.welcome),
      ),
    };
  }

  Widget _restoring() {
    return const Scaffold(
      body: AsyncStatePanel(
        icon: Icons.key,
        title: 'Restoring account setup',
        message: 'Loading your unfinished Nostr account securely.',
      ),
    );
  }

  Widget _recoveredProfile(
    BuildContext context,
    AccountCreationProfileRecovery recovery,
  ) {
    return CreateAccountProfileScreen(
      initial: null,
      onSubmit: context.read<AccountCreationCubit>().recoverProfile,
      isSubmitting: recovery.isSubmitting,
      errorMessage: recovery.message,
    );
  }

  Widget _backup(
    BuildContext context,
    AuthSecret secret,
    bool isFinishing,
    String? message, {
    bool canSkipPicture = false,
  }) {
    return PrivateKeyBackupScreen(
      secret: secret,
      isFinishing: isFinishing,
      errorMessage: message,
      onCopy: () => widget.secretBackup.copy(secret),
      onSkipPicture: canSkipPicture
          ? context.read<AccountCreationCubit>().skipPicture
          : null,
      onFinish: context.read<AccountCreationCubit>().complete,
    );
  }

  void _show(_AccessPage page) => setState(() => _page = page);

  SelectedProfileImage? _selectedPicture(AccountCreationState state) {
    return switch (state) {
      AccountCreationIdle(:final selectedPicture) => selectedPicture,
      AccountCreationSelectingPicture(:final selectedPicture) =>
        selectedPicture,
      _ => null,
    };
  }

  void _acceptCreated(BuildContext context, AccountCreationState state) {
    final completed = state as AccountCreationCompleted;
    context.read<SessionCubit>().acceptCreatedSession(completed.session);
  }
}
