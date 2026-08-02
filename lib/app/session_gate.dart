import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/home_shell.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';
import 'package:ghostr/features/session/presentation/sign_in_screen.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';

class SessionGate extends StatelessWidget {
  const SessionGate({required this.controllers, super.key});

  final AppControllerFactory controllers;

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<SessionCubit, SessionState>(
      listenWhen: (_, current) =>
          current is SessionSignedIn && current.errorMessage != null,
      listener: _showSignedInError,
      builder: _content,
    );
  }

  Widget _content(BuildContext context, SessionState state) {
    return switch (state) {
      SessionLoading() => _loading(),
      SessionSigningOut() => _signingOut(),
      SessionSignedOut(errorMessage: final message) => SignInScreen(
          errorMessage: message,
          onSubmit: context.read<SessionCubit>().signIn,
        ),
      SessionRestoreFailure(message: final message) => _failure(
          context,
          message,
        ),
      SessionSignedIn(session: final session) =>
        HomeShell(session: session, controllers: controllers),
    };
  }

  Widget _loading() {
    return const Scaffold(
      body: AsyncStatePanel(
        icon: Icons.bolt,
        title: 'Booting Ghostr',
        message: 'Preparing your Nostr-powered video stack.',
      ),
    );
  }

  Widget _signingOut() {
    return const Scaffold(
      body: AsyncStatePanel(
        icon: Icons.logout,
        title: 'Signing out',
        message: 'Removing your Nostr key from this device.',
      ),
    );
  }

  Widget _failure(BuildContext context, String message) {
    return Scaffold(
      body: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          AsyncStatePanel(
            icon: Icons.lock_outline,
            title: 'Session unavailable',
            message: message,
            actionLabel: 'Retry',
            onAction: context.read<SessionCubit>().restore,
          ),
          TextButton(
            onPressed: context.read<SessionCubit>().resetStoredSession,
            child: const Text('Use another key'),
          ),
        ],
      ),
    );
  }

  void _showSignedInError(BuildContext context, SessionState state) {
    final signedIn = state as SessionSignedIn;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(signedIn.errorMessage!)),
    );
    context.read<SessionCubit>().clearError();
  }
}
