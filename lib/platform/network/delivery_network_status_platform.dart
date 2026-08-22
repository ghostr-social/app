import 'package:flutter/foundation.dart';
import 'package:ghostr/core/network/delivery_network_status_port.dart';
import 'package:ghostr/platform/network/android_delivery_network_status.dart';
import 'package:ghostr/platform/network/unavailable_delivery_network_status.dart';

DeliveryNetworkStatusPort currentDeliveryNetworkStatusPlatform() {
  if (!kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
    return AndroidDeliveryNetworkStatus();
  }
  return const UnavailableDeliveryNetworkStatus();
}
