import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/profile_content.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

class ProfileScreen extends StatelessWidget {
  const ProfileScreen({
    required this.onSignedOut,
    this.onOpenSettings,
    super.key,
  });

  final VoidCallback? onOpenSettings;
  final VoidCallback onSignedOut;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Profile'),
        actions: [_settingsAction()],
      ),
      body: BlocConsumer<ProfileCubit, ProfileState>(
        listenWhen: (_, state) => state.notice != null,
        listener: _showNotice,
        builder: _body,
      ),
    );
  }

  Widget _settingsAction() {
    final callback = onOpenSettings;
    if (callback == null) return const SizedBox.shrink();
    return IconButton(
      tooltip: 'Open settings',
      onPressed: callback,
      icon: const Icon(Icons.settings_outlined),
    );
  }

  Widget _body(BuildContext context, ProfileState state) {
    return switch (state.status) {
      ProfileStatus.loading =>
        const LoadingPanel(label: 'Loading creator profile'),
      ProfileStatus.failure => _errorPanel(context, state.message!),
      ProfileStatus.ready => ProfileContent(
          details: state.details!,
          isUpdating: state.isUpdating,
          actions: ProfileContentActions(
            onFollow: (_) => context.read<ProfileCubit>().toggleFollow(),
            onBlock: (_) => context.read<ProfileCubit>().toggleBlock(),
            onSignOut: onSignedOut,
          ),
        ),
    };
  }

  Widget _errorPanel(BuildContext context, String message) {
    return AsyncStatePanel(
      icon: Icons.person_off,
      title: 'Profile unavailable',
      message: message,
      actionLabel: 'Retry',
      onAction: context.read<ProfileCubit>().load,
    );
  }

  void _showNotice(BuildContext context, ProfileState state) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(state.notice!)),
    );
    context.read<ProfileCubit>().clearNotice();
  }
}
