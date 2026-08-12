import 'package:flutter/widgets.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ghostr/features/settings/domain/app_update_preferences.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/settings/domain/video_inventory_budget.dart';

class SettingsFormActions {
  const SettingsFormActions({
    required this.relays,
    required this.searchRelays,
    required this.blossom,
    required this.onBudgetChanged,
    required this.onDataUsageChanged,
    required this.onHideWatchedChanged,
    required this.updates,
    required this.onSave,
    this.onOpenWatchHistory,
  });

  final RelaySettingsActions relays;
  final RelaySettingsActions searchRelays;
  final BlossomSettingsActions blossom;
  final ValueChanged<VideoInventoryBudget> onBudgetChanged;
  final ValueChanged<DataUsageLevel> onDataUsageChanged;
  final ValueChanged<bool> onHideWatchedChanged;
  final UpdateSettingsActions updates;
  final VoidCallback onSave;
  final VoidCallback? onOpenWatchHistory;
}

class UpdateSettingsActions {
  const UpdateSettingsActions({
    required this.onChanged,
    required this.onCheckNow,
  });

  final ValueChanged<AppUpdatePreferences> onChanged;
  final VoidCallback? onCheckNow;
}

class RelaySettingsActions {
  const RelaySettingsActions({required this.onAdd, required this.onRemove});

  final VoidCallback onAdd;
  final ValueChanged<RelayUrl> onRemove;
}

class BlossomSettingsActions {
  const BlossomSettingsActions({required this.onAdd, required this.onRemove});

  final VoidCallback onAdd;
  final ValueChanged<BlossomServerUrl> onRemove;
}
