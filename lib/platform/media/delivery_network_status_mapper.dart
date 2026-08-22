import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/src/rust/api/network_control.dart';

FfiDeliveryNetworkStatus ffiDeliveryNetworkStatus(
  DeliveryNetworkStatus status,
) {
  return FfiDeliveryNetworkStatus(
    networkClass: _networkClass(status.networkClass),
    generation: BigInt.from(status.generation),
  );
}

FfiDeliveryNetworkClass _networkClass(DeliveryNetworkClass networkClass) {
  return switch (networkClass) {
    DeliveryNetworkClass.unavailable => FfiDeliveryNetworkClass.unavailable,
    DeliveryNetworkClass.wifi => FfiDeliveryNetworkClass.wifi,
    DeliveryNetworkClass.cellular => FfiDeliveryNetworkClass.cellular,
    DeliveryNetworkClass.wired => FfiDeliveryNetworkClass.wired,
    DeliveryNetworkClass.constrained => FfiDeliveryNetworkClass.constrained,
  };
}
