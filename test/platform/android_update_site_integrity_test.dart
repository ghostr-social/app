import 'package:flutter_test/flutter_test.dart';

import '../support/update_site_sync_harness.dart';

void main() {
  test('rejects a public manifest with matching version but wrong bytes', () {
    final harness = UpdateSiteSyncHarness.create();
    addTearDown(harness.dispose);

    final result = harness.run(corruptAlways: true);

    expect(result.exitCode, 69);
    expect(result.stderr, contains('did not publish'));
    expect(harness.curlCalls, hasLength(2));
  });
}
