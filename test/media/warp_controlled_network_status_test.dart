import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/network/delivery_network_status.dart';

import '../../integration_test/support/warp_controlled_network_status.dart';

void main() {
  test(
    'controlled WARP network emits strictly ordered status generations',
    () async {
      final network = WarpControlledNetworkStatus();
      addTearDown(network.close);
      final observed = <DeliveryNetworkStatus>[];
      final subscription = network.changes.listen(observed.add);
      addTearDown(subscription.cancel);

      expect(
        await network.read(),
        const DeliveryNetworkStatus(DeliveryNetworkClass.wifi, generation: 1),
      );
      final constrained = network.publish(DeliveryNetworkClass.constrained);
      final recovered = network.publish(DeliveryNetworkClass.wifi);

      expect(constrained.generation, 2);
      expect(recovered.generation, 3);
      expect(observed, [constrained, recovered]);
    },
  );
}
