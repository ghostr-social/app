import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('the confirmed window fence follows the latest flush', () {
    expect(progressiveConfirmedWindowFence([1200, 1800, 1400]), 1800);
  });
}
