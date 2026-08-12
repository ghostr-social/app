import 'package:flutter_test/flutter_test.dart';

import '../support/release_publisher_harness.dart';

void main() {
  test('refuses to upload when an existing tag targets another commit', () {
    final harness = ReleasePublisherHarness.create(
      releaseExists: true,
      tagCommit: 'cafebabe',
    );
    try {
      final result = harness.run(target: 'deadbeef');

      expect(result.exitCode, isNot(0));
      expect(result.stderr, contains('does not match built commit'));
      expect(harness.calls, [
        'api repos/ghostr-social/app/commits/v1.2.3 --jq .sha',
      ]);
    } finally {
      harness.dispose();
    }
  });
}
