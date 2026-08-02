import 'package:flutter_test/flutter_test.dart';

import '../support/release_publisher_harness.dart';

void main() {
  test('creates a missing tag release before uploading APKs', () {
    final harness = ReleasePublisherHarness.create(releaseExists: false);
    try {
      final result = harness.run();

      expect(result.exitCode, 0, reason: '${result.stdout}\n${result.stderr}');
      expect(harness.calls, [
        'release view v1.2.3',
        'release create v1.2.3 --title Release v1.2.3 --generate-notes',
        'release upload v1.2.3 ${harness.assets.join(' ')} --clobber',
      ]);
    } finally {
      harness.dispose();
    }
  });
}
