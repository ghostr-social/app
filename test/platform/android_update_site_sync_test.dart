import 'package:flutter_test/flutter_test.dart';

import '../support/update_site_sync_harness.dart';

void main() {
  test('dispatches the site workflow and waits for the published version', () {
    final harness = UpdateSiteSyncHarness.create();
    addTearDown(harness.dispose);

    final result = harness.run();

    expect(result.exitCode, 0, reason: '${result.stdout}\n${result.stderr}');
    expect(harness.ghCalls, [
      'workflow run sync_stable.yml '
          '--repo ghostr-social/ghostr-social.github.io --ref main',
    ]);
    expect(harness.curlCalls, hasLength(2));
    expect(harness.curlCalls.last, contains('--connect-timeout 5'));
    expect(harness.curlCalls.last, contains('--max-time 15'));
  });
}
