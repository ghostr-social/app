import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/core/network/delivery_network_status_port.dart';
import 'package:ghostr/platform/media/delivery_network_status_runtime.dart';

void main() {
  test('forwards only fresh authoritative transport generations', () async {
    final port = _NetworkPort(
      const DeliveryNetworkStatus(DeliveryNetworkClass.wifi, generation: 3),
    );
    final applied = <DeliveryNetworkStatus>[];
    var rejectCellularOnce = true;
    final runtime = await DeliveryNetworkStatusRuntime.start(
      port: port,
      initial: const DeliveryNetworkStatus(
        DeliveryNetworkClass.unavailable,
        generation: 0,
      ),
      apply: (status) async {
        applied.add(status);
        if (status.networkClass == DeliveryNetworkClass.cellular &&
            rejectCellularOnce) {
          rejectCellularOnce = false;
          return false;
        }
        return true;
      },
    );
    addTearDown(runtime.close);

    port.emit(
      const DeliveryNetworkStatus(DeliveryNetworkClass.cellular, generation: 4),
    );
    port.emit(
      const DeliveryNetworkStatus(DeliveryNetworkClass.wifi, generation: 3),
    );
    await pumpEventQueue();
    await runtime.settled;

    expect(applied, [
      const DeliveryNetworkStatus(DeliveryNetworkClass.wifi, generation: 3),
      const DeliveryNetworkStatus(DeliveryNetworkClass.cellular, generation: 4),
      const DeliveryNetworkStatus(DeliveryNetworkClass.cellular, generation: 4),
    ]);
  });
}

final class _NetworkPort implements DeliveryNetworkStatusPort {
  _NetworkPort(this.current);

  DeliveryNetworkStatus current;
  final _changes = StreamController<DeliveryNetworkStatus>.broadcast();

  @override
  Stream<DeliveryNetworkStatus> get changes => _changes.stream;

  void emit(DeliveryNetworkStatus value) {
    current = value;
    _changes.add(value);
  }

  @override
  Future<DeliveryNetworkStatus> read() async => current;

  @override
  Future<void> close() => _changes.close();
}
