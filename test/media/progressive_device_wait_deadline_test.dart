import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_wait.dart';

void main() {
  test('device conditions fail before the gateway idle deadline', () {
    expect(
      progressiveDeviceConditionTimeout,
      lessThan(const Duration(seconds: 15)),
    );
  });
}
