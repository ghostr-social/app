import 'package:flutter/material.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';
import 'package:ghostr/features/app_update/presentation/app_update_status_panel.dart';
import 'package:ghostr/features/app_update/presentation/app_update_state.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class SettingsUpdateSection extends StatelessWidget {
  const SettingsUpdateSection({
    required this.preferences,
    required this.isSaving,
    required this.onChanged,
    required this.onCheckNow,
    this.updateState,
    this.updateActions,
    super.key,
  });

  final AppUpdatePreferences preferences;
  final bool isSaving;
  final ValueChanged<AppUpdatePreferences> onChanged;
  final VoidCallback? onCheckNow;
  final AppUpdateState? updateState;
  final AppUpdateStatusActions? updateActions;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text('App updates', style: Theme.of(context).textTheme.titleLarge),
        const SizedBox(height: AppSpacing.xs),
        const Text('Get signed Android releases from ghostr.social.'),
        const SizedBox(height: AppSpacing.sm),
        SwitchListTile(
          key: const Key('automatic-update-checks-field'),
          title: const Text('Offer new app versions automatically'),
          subtitle: const Text(
            'Checks quietly; skipped versions are not offered again.',
          ),
          value: preferences.automaticChecks,
          onChanged: isSaving ? null : _changeAutomaticChecks,
        ),
        SwitchListTile(
          key: const Key('wifi-only-update-downloads-field'),
          title: const Text('Download updates only on Wi-Fi'),
          subtitle: const Text(
            'When off, downloads can use mobile data after you tap Update.',
          ),
          value: preferences.downloadPolicy == UpdateDownloadPolicy.wifiOnly,
          onChanged: isSaving ? null : _changeWifiOnly,
        ),
        SwitchListTile(
          key: const Key('automatic-update-install-field'),
          title: const Text('Install updates automatically'),
          subtitle: const Text('Android may still ask you to confirm.'),
          value: preferences.automaticInstall,
          onChanged: isSaving ? null : _changeAutomaticInstall,
        ),
        OutlinedButton.icon(
          key: const Key('check-for-updates-button'),
          onPressed: isSaving ? null : onCheckNow,
          icon: const Icon(Icons.system_update),
          label: const Text('Check now'),
        ),
        if (updateState case final state?) ...[
          const SizedBox(height: AppSpacing.sm),
          AppUpdateStatusPanel(state: state, actions: updateActions!),
        ],
      ],
    );
  }

  void _changeAutomaticChecks(bool value) {
    onChanged(preferences.copyWith(automaticChecks: value));
  }

  void _changeWifiOnly(bool value) {
    onChanged(
      preferences.copyWith(
        downloadPolicy: value
            ? UpdateDownloadPolicy.wifiOnly
            : UpdateDownloadPolicy.anyNetwork,
      ),
    );
  }

  void _changeAutomaticInstall(bool value) {
    onChanged(preferences.copyWith(automaticInstall: value));
  }
}
