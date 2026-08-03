import 'package:flutter/widgets.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/settings/domain/video_inventory_budget.dart';

class SettingsFormActions {
  const SettingsFormActions({
    required this.relays,
    required this.blossom,
    required this.onBudgetChanged,
    required this.onHideWatchedChanged,
    required this.onSave,
    this.onOpenWatchHistory,
  });

  final RelaySettingsActions relays;
  final BlossomSettingsActions blossom;
  final ValueChanged<VideoInventoryBudget> onBudgetChanged;
  final ValueChanged<bool> onHideWatchedChanged;
  final VoidCallback onSave;
  final VoidCallback? onOpenWatchHistory;
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
