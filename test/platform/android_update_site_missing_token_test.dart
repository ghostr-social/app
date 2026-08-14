import 'package:flutter_test/flutter_test.dart';

import '../support/update_site_sync_harness.dart';

void main() {
  test('fails without invoking tools when the dispatch token is missing', () {
    final harness = UpdateSiteSyncHarness.create();
    addTearDown(harness.dispose);

    final result = harness.run(missingToken: true);

    expect(result.exitCode, isNonZero);
    expect(result.stderr, contains('UPDATE_SITE_TOKEN'));
    expect(harness.ghLog.existsSync(), isFalse);
    expect(harness.curlLog.existsSync(), isFalse);
  });
}
