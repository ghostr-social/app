import 'package:flutter/material.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/settings/presentation/settings_form_actions.dart';
import 'package:ghostr/features/settings/presentation/settings_update_section.dart';
import 'package:ghostr/features/settings/presentation/settings_watch_history_section.dart';
import 'package:ghostr/features/app_update/presentation/app_update_status_panel.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class SettingsForm extends StatelessWidget {
  const SettingsForm({
    required this.settings,
    required this.isSaving,
    required this.actions,
    this.updateState,
    this.updateActions,
    super.key,
  });

  final AppSettings settings;
  final bool isSaving;
  final SettingsFormActions actions;
  final AppUpdateState? updateState;
  final AppUpdateStatusActions? updateActions;

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(AppSpacing.lg),
      children: [
        ..._relaySection(context),
        ..._searchRelaySection(context),
        ..._blossomSection(context),
        ..._inventorySection(context),
        const SizedBox(height: AppSpacing.xxl),
        ..._dataUsageSection(context),
        const SizedBox(height: AppSpacing.xxl),
        SettingsUpdateSection(
          preferences: settings.updatePreferences,
          isSaving: isSaving,
          onChanged: actions.updates.onChanged,
          onCheckNow: actions.updates.onCheckNow,
          updateState: updateState,
          updateActions: updateActions,
        ),
        const SizedBox(height: AppSpacing.xxl),
        SettingsWatchHistorySection(
          hideWatchedVideos: settings.hideWatchedVideos,
          isSaving: isSaving,
          onHideWatchedChanged: actions.onHideWatchedChanged,
          onOpenWatchHistory: actions.onOpenWatchHistory,
        ),
        const SizedBox(height: AppSpacing.xl),
        _saveButton(),
      ],
    );
  }

  List<Widget> _relaySection(BuildContext context) {
    return [
      _sectionTitle(context, 'Relay connections'),
      const SizedBox(height: AppSpacing.xs),
      const Text('Choose which Nostr relays Ghostr reads and writes.'),
      const SizedBox(height: AppSpacing.sm),
      ...settings.relays.map(
        (relay) => _relayTile(relay, actions.relays.onRemove),
      ),
      _addButton('Add relay', actions.relays.onAdd),
      const SizedBox(height: AppSpacing.xxl),
    ];
  }

  List<Widget> _searchRelaySection(BuildContext context) {
    return [
      _sectionTitle(context, 'Search relays'),
      const SizedBox(height: AppSpacing.xs),
      const Text('Relays with NIP-50 search that power discovery.'),
      const SizedBox(height: AppSpacing.sm),
      ...settings.searchRelays.map(
        (relay) => _relayTile(relay, actions.searchRelays.onRemove),
      ),
      _addButton('Add search relay', actions.searchRelays.onAdd),
      const SizedBox(height: AppSpacing.xxl),
    ];
  }

  List<Widget> _dataUsageSection(BuildContext context) {
    return [
      _sectionTitle(context, 'Data usage'),
      const SizedBox(height: AppSpacing.xs),
      const Text('Control how aggressively Ghostr fetches content.'),
      const SizedBox(height: AppSpacing.sm),
      DropdownButtonFormField<DataUsageLevel>(
        key: const Key('data-usage-field'),
        initialValue: settings.dataUsage,
        decoration: const InputDecoration(labelText: 'Network activity'),
        items: DataUsageLevel.values.map(_dataUsageItem).toList(),
        onChanged: isSaving ? null : _changeDataUsage,
      ),
    ];
  }

  List<Widget> _blossomSection(BuildContext context) {
    return [
      _sectionTitle(context, 'Media uploads'),
      const SizedBox(height: AppSpacing.xs),
      const Text('Choose the Blossom servers that host published videos.'),
      const SizedBox(height: AppSpacing.sm),
      ...settings.blossomServers.map(_blossomTile),
      _addButton('Add media server', actions.blossom.onAdd),
      const SizedBox(height: AppSpacing.xxl),
    ];
  }

  List<Widget> _inventorySection(BuildContext context) {
    return [
      _sectionTitle(context, 'Video inventory'),
      const SizedBox(height: AppSpacing.xs),
      const Text('Set the on-device budget for videos prepared ahead.'),
      const SizedBox(height: AppSpacing.sm),
      DropdownButtonFormField<VideoInventoryBudget>(
        key: const Key('inventory-budget-field'),
        initialValue: settings.inventoryBudget,
        decoration: const InputDecoration(labelText: 'Storage budget'),
        items: VideoInventoryBudget.values.map(_budgetItem).toList(),
        onChanged: isSaving ? null : _changeBudget,
      ),
    ];
  }

  Widget _sectionTitle(BuildContext context, String title) {
    return Text(title, style: Theme.of(context).textTheme.titleLarge);
  }

  Widget _addButton(String label, VoidCallback onPressed) {
    return OutlinedButton.icon(
      onPressed: isSaving ? null : onPressed,
      icon: const Icon(Icons.add),
      label: Text(label),
    );
  }

  void _changeBudget(VideoInventoryBudget? value) {
    if (value != null) actions.onBudgetChanged(value);
  }

  void _changeDataUsage(DataUsageLevel? value) {
    if (value != null) actions.onDataUsageChanged(value);
  }

  DropdownMenuItem<DataUsageLevel> _dataUsageItem(DataUsageLevel level) {
    return DropdownMenuItem(value: level, child: Text(level.label));
  }

  Widget _saveButton() {
    return ElevatedButton(
      key: const Key('save-settings-button'),
      onPressed: isSaving ? null : actions.onSave,
      child: Text(isSaving ? 'Saving...' : 'Save settings'),
    );
  }

  Widget _relayTile(RelayUrl relay, ValueChanged<RelayUrl> onRemove) {
    return Card(
      child: ListTile(
        title: Text(relay.value),
        trailing: IconButton(
          tooltip: 'Remove ${relay.value}',
          onPressed: isSaving ? null : () => onRemove(relay),
          icon: const Icon(Icons.delete_outline),
        ),
      ),
    );
  }

  Widget _blossomTile(BlossomServerUrl server) {
    return Card(
      child: ListTile(
        title: Text(server.value),
        trailing: IconButton(
          tooltip: 'Remove ${server.value}',
          onPressed: isSaving ? null : () => actions.blossom.onRemove(server),
          icon: const Icon(Icons.delete_outline),
        ),
      ),
    );
  }

  DropdownMenuItem<VideoInventoryBudget> _budgetItem(
    VideoInventoryBudget budget,
  ) {
    return DropdownMenuItem(value: budget, child: Text(budget.label));
  }
}
