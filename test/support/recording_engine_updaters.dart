import 'package:ghostr/src/rust/api/delivery_types.dart';
import 'package:ghostr/src/rust/api/focus_control.dart';
import 'package:ghostr/platform/media/rust_engine_configuration.dart';

/// Records `ffi_update_focus` payloads handed to the Rust engine; set
/// [failure] to make the next call throw instead.
final class RecordingRustFocusUpdater {
  final List<RecordedFocusUpdate> updates = [];
  Object? failure;

  Future<void> call({required FfiFocusUpdate update}) async {
    final error = failure;
    if (error != null) throw error;
    updates.add(
      RecordedFocusUpdate(
        update.feedId,
        update.items,
        update.currentIndex,
        update.watchMs,
        update.generation,
      ),
    );
  }
}

final class RecordedFocusUpdate {
  RecordedFocusUpdate(
    this.feedId,
    this.items,
    this.currentIndex,
    this.watchMs,
    this.generation,
  );

  final String feedId;
  final List<FfiFocusItem> items;
  final int currentIndex;
  final BigInt watchMs;
  final BigInt generation;
}

/// Records `ffi_set_delivery_config` pushes to the Rust engine; set
/// [failure] to make the next push throw instead.
final class RecordingDeliveryConfigUpdater {
  final List<RustEngineConfiguration> pushes = [];
  Object? failure;

  Future<void> call(RustEngineConfiguration configuration) async {
    final error = failure;
    if (error != null) throw error;
    pushes.add(configuration);
  }
}
