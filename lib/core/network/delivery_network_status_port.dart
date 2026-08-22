import 'package:ghostr/core/network/delivery_network_status.dart';

abstract interface class DeliveryNetworkStatusPort {
  Future<DeliveryNetworkStatus> read();

  Stream<DeliveryNetworkStatus> get changes;

  Future<void> close();
}
