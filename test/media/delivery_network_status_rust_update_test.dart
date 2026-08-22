import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/platform/media/delivery_network_status_runtime.dart';
import 'package:ghostr/src/rust/api/network_control.dart';

void main() {
  test('forwards the mapped delivery status to the Rust updater', () async {
    FfiDeliveryNetworkStatus? received;

    final updated = await updateRustDeliveryNetworkStatus(
      const DeliveryNetworkStatus(
        DeliveryNetworkClass.constrained,
        generation: 11,
      ),
      update: ({required status}) async {
        received = status;
        return true;
      },
    );

    expect(updated, isTrue);
    expect(
      received,
      FfiDeliveryNetworkStatus(
        networkClass: FfiDeliveryNetworkClass.constrained,
        generation: BigInt.from(11),
      ),
    );
  });
}
