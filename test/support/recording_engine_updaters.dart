import 'package:ghostr/src/rust/api/delivery_types.dart';

/// Records `ffi_update_focus` payloads handed to the Rust engine; set
/// [failure] to make the next call throw instead.
final class RecordingRustFocusUpdater {
  final List<RecordedFocusUpdate> updates = [];
  Object? failure;

  Future<void> call({
    required String feedId,
    required List<FfiFocusItem> items,
    required int currentIndex,
    required BigInt watchMs,
  }) async {
    final error = failure;
    if (error != null) throw error;
    updates.add(RecordedFocusUpdate(feedId, items, currentIndex, watchMs));
  }
}

final class RecordedFocusUpdate {
  RecordedFocusUpdate(this.feedId, this.items, this.currentIndex, this.watchMs);

  final String feedId;
  final List<FfiFocusItem> items;
  final int currentIndex;
  final BigInt watchMs;
}

/// Records `ffi_set_delivery_config` pushes to the Rust engine; set
/// [failure] to make the next push throw instead.
final class RecordingDeliveryConfigUpdater {
  final List<(String, BigInt)> pushes = [];
  Object? failure;

  Future<void> call({
    required String dataUsage,
    required BigInt maxStorageBytes,
  }) async {
    final error = failure;
    if (error != null) throw error;
    pushes.add((dataUsage, maxStorageBytes));
  }
}
