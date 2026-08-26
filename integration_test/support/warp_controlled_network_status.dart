import 'dart:async';

import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/core/network/delivery_network_status_port.dart';

final class WarpControlledNetworkStatus implements DeliveryNetworkStatusPort {
  final _changes = StreamController<DeliveryNetworkStatus>.broadcast(
    sync: true,
  );
  var _current = const DeliveryNetworkStatus(
    DeliveryNetworkClass.wifi,
    generation: 1,
  );

  @override
  Future<DeliveryNetworkStatus> read() async => _current;

  @override
  Stream<DeliveryNetworkStatus> get changes => _changes.stream;

  DeliveryNetworkStatus publish(DeliveryNetworkClass networkClass) {
    _current = DeliveryNetworkStatus(
      networkClass,
      generation: _current.generation + 1,
    );
    _changes.add(_current);
    return _current;
  }

  @override
  Future<void> close() => _changes.close();
}
