import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/core/network/delivery_network_status_port.dart';

final class UnavailableDeliveryNetworkStatus
    implements DeliveryNetworkStatusPort {
  const UnavailableDeliveryNetworkStatus();

  @override
  Stream<DeliveryNetworkStatus> get changes => const Stream.empty();

  @override
  Future<DeliveryNetworkStatus> read() async {
    return DeliveryNetworkStatus.unavailable;
  }

  @override
  Future<void> close() async {}
}
