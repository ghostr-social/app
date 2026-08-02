import 'package:flutter/material.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/settings/presentation/settings_form_actions.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class SettingsForm extends StatelessWidget {
  const SettingsForm({
    required this.settings,
    required this.isSaving,
    required this.actions,
    super.key,
  });

  final AppSettings settings;
  final bool isSaving;
  final SettingsFormActions actions;

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(AppSpacing.lg),
      children: [
        ..._relaySection(context),
        ..._blossomSection(context),
        ..._inventorySection(context),
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
      ...settings.relays.map(_relayTile),
      _addButton('Add relay', actions.relays.onAdd),
      const SizedBox(height: AppSpacing.xxl),
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

  Widget _saveButton() {
    return ElevatedButton(
      onPressed: isSaving ? null : actions.onSave,
      child: Text(isSaving ? 'Saving...' : 'Save settings'),
    );
  }

  Widget _relayTile(RelayUrl relay) {
    return Card(
      child: ListTile(
        title: Text(relay.value),
        trailing: IconButton(
          tooltip: 'Remove ${relay.value}',
          onPressed: isSaving ? null : () => actions.relays.onRemove(relay),
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
