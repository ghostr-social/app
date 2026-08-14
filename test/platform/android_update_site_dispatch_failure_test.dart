import 'package:flutter_test/flutter_test.dart';

import '../support/update_site_sync_harness.dart';

void main() {
  test('fails before polling when the site dispatch is rejected', () {
    final harness = UpdateSiteSyncHarness.create();
    addTearDown(harness.dispose);

    final result = harness.run(failDispatch: true);

    expect(result.exitCode, 1);
    expect(harness.ghCalls, hasLength(1));
    expect(harness.curlLog.existsSync(), isFalse);
  });
}
