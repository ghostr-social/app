import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/platform/media/delivery_network_status_mapper.dart';
import 'package:ghostr/src/rust/api/network_control.dart';

void main() {
  test('maps every delivery network class into the Rust boundary', () {
    const classes = <DeliveryNetworkClass, FfiDeliveryNetworkClass>{
      DeliveryNetworkClass.unavailable: FfiDeliveryNetworkClass.unavailable,
      DeliveryNetworkClass.wifi: FfiDeliveryNetworkClass.wifi,
      DeliveryNetworkClass.cellular: FfiDeliveryNetworkClass.cellular,
      DeliveryNetworkClass.wired: FfiDeliveryNetworkClass.wired,
      DeliveryNetworkClass.constrained: FfiDeliveryNetworkClass.constrained,
    };

    for (final entry in classes.entries) {
      final mapped = ffiDeliveryNetworkStatus(
        DeliveryNetworkStatus(entry.key, generation: 9),
      );

      expect(mapped.networkClass, entry.value);
      expect(mapped.generation, BigInt.from(9));
    }
  });
}
