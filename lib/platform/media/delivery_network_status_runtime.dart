import 'dart:async';

import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/core/network/delivery_network_status_port.dart';
import 'package:ghostr/platform/media/delivery_network_status_mapper.dart';
import 'package:ghostr/src/rust/api/network_control.dart';

typedef DeliveryNetworkStatusApplier =
    Future<bool> Function(DeliveryNetworkStatus status);
typedef RustDeliveryNetworkStatusUpdater =
    Future<bool> Function({required FfiDeliveryNetworkStatus status});

Future<bool> updateRustDeliveryNetworkStatus(
  DeliveryNetworkStatus status, {
  RustDeliveryNetworkStatusUpdater update = ffiSetDeliveryNetwork,
}) {
  return update(status: ffiDeliveryNetworkStatus(status));
}

final class DeliveryNetworkStatusRuntime {
  static const _maximumApplyAttempts = 2;

  DeliveryNetworkStatusRuntime._({
    required DeliveryNetworkStatusPort port,
    required DeliveryNetworkStatus initial,
    required DeliveryNetworkStatusApplier apply,
  }) : _port = port,
       _latest = initial,
       _apply = apply;

  final DeliveryNetworkStatusPort _port;
  final DeliveryNetworkStatusApplier _apply;
  DeliveryNetworkStatus _latest;
  StreamSubscription<DeliveryNetworkStatus>? _subscription;
  Future<void> _serial = Future.value();
  bool _closed = false;

  static Future<DeliveryNetworkStatusRuntime> start({
    required DeliveryNetworkStatusPort port,
    required DeliveryNetworkStatus initial,
    required DeliveryNetworkStatusApplier apply,
  }) async {
    final runtime = DeliveryNetworkStatusRuntime._(
      port: port,
      initial: initial,
      apply: apply,
    );
    runtime._subscription = port.changes.listen(runtime._enqueue);
    await runtime._refresh();
    await runtime.settled;
    return runtime;
  }

  Future<void> get settled => _serial;

  Future<void> close() async {
    if (_closed) return;
    _closed = true;
    await _subscription?.cancel();
    await settled;
    await _port.close();
  }

  void _enqueue(DeliveryNetworkStatus status) {
    if (_closed || !status.isFresherThan(_latest)) return;
    _serial = _serial.then((_) => _applyIfFresh(status));
  }

  Future<void> _applyIfFresh(DeliveryNetworkStatus status) async {
    if (!status.isFresherThan(_latest)) return;
    for (var attempt = 0; attempt < _maximumApplyAttempts; attempt++) {
      try {
        if (!await _apply(status)) continue;
        _latest = status;
        return;
      } on Object catch (error, stackTrace) {
        _logApplyFailure(error, stackTrace);
      }
    }
  }

  void _logApplyFailure(Object error, StackTrace stackTrace) {
    logBoundaryFailure(
      source: 'ghostr.video.network',
      message: 'The delivery network class could not be updated.',
      error: error,
      stackTrace: stackTrace,
    );
  }

  Future<void> _refresh() async {
    try {
      _enqueue(await _port.read());
    } on Object catch (error, stackTrace) {
      logBoundaryFailure(
        source: 'ghostr.video.network',
        message: 'The delivery network class is unavailable.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
