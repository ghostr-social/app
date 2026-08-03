import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/settings/presentation/settings_cubit.dart';
import 'package:ghostr/features/settings/presentation/settings_form.dart';
import 'package:ghostr/features/settings/presentation/settings_form_actions.dart';
import 'package:ghostr/features/settings/presentation/settings_url_dialog.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({this.onOpenWatchHistory, super.key});

  final VoidCallback? onOpenWatchHistory;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: BlocConsumer<SettingsCubit, SettingsState>(
        listenWhen: (_, state) => state.notice != null,
        listener: _showNotice,
        builder: _body,
      ),
    );
  }

  Widget _body(BuildContext context, SettingsState state) {
    return switch (state) {
      SettingsLoading() => const LoadingPanel(label: 'Loading settings'),
      SettingsFailure(:final failureMessage) =>
        _errorPanel(context, failureMessage),
      SettingsReady(:final readySettings, :final isSaving) => SettingsForm(
          settings: readySettings,
          isSaving: isSaving,
          actions: _formActions(context),
        ),
    };
  }

  Widget _errorPanel(BuildContext context, String message) {
    return AsyncStatePanel(
      icon: Icons.settings_backup_restore,
      title: 'Settings unavailable',
      message: message,
      actionLabel: 'Retry',
      onAction: context.read<SettingsCubit>().load,
    );
  }

  SettingsFormActions _formActions(BuildContext context) {
    final cubit = context.read<SettingsCubit>();
    return SettingsFormActions(
      relays: RelaySettingsActions(
        onAdd: () => _addRelay(context),
        onRemove: cubit.removeRelay,
      ),
      searchRelays: RelaySettingsActions(
        onAdd: () => _addSearchRelay(context),
        onRemove: cubit.removeSearchRelay,
      ),
      blossom: BlossomSettingsActions(
        onAdd: () => _addBlossomServer(context),
        onRemove: cubit.removeBlossomServer,
      ),
      onBudgetChanged: cubit.changeBudget,
      onDataUsageChanged: cubit.changeDataUsage,
      onHideWatchedChanged: cubit.changeHideWatchedVideos,
      onSave: cubit.save,
      onOpenWatchHistory: onOpenWatchHistory,
    );
  }

  Future<void> _addSearchRelay(BuildContext context) async {
    final value = await _showUrlDialog(
      context,
      const SettingsUrlDialogRequest(
        title: 'Add search relay',
        fieldKey: Key('search-relay-url-field'),
        hintText: 'wss://search.example',
      ),
    );
    if (context.mounted && value != null) {
      context.read<SettingsCubit>().addSearchRelay(value);
    }
  }

  Future<void> _addRelay(BuildContext context) async {
    final value = await _showUrlDialog(
      context,
      const SettingsUrlDialogRequest(
        title: 'Add relay',
        fieldKey: Key('relay-url-field'),
        hintText: 'wss://relay.example',
      ),
    );
    if (context.mounted && value != null) {
      context.read<SettingsCubit>().addRelay(value);
    }
  }

  Future<void> _addBlossomServer(BuildContext context) async {
    final value = await _showUrlDialog(
      context,
      const SettingsUrlDialogRequest(
        title: 'Add media server',
        fieldKey: Key('blossom-server-url-field'),
        hintText: 'https://blossom.example',
      ),
    );
    if (context.mounted && value != null) {
      context.read<SettingsCubit>().addBlossomServer(value);
    }
  }

  Future<String?> _showUrlDialog(
    BuildContext context,
    SettingsUrlDialogRequest request,
  ) {
    return showSettingsUrlDialog(context, request);
  }

  void _showNotice(BuildContext context, SettingsState state) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(state.notice!)),
    );
    context.read<SettingsCubit>().clearNotice();
  }
}
