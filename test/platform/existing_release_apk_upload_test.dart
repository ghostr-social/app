import 'package:flutter_test/flutter_test.dart';

import '../support/release_publisher_harness.dart';

void main() {
  test('uploads APKs when the tag release already exists', () {
    final harness = ReleasePublisherHarness.create(releaseExists: true);
    try {
      final result = harness.run();

      expect(result.exitCode, 0, reason: '${result.stdout}\n${result.stderr}');
      expect(harness.calls, [
        'release view v1.2.3',
        'release upload v1.2.3 ${harness.assets.join(' ')} --clobber',
      ]);
    } finally {
      harness.dispose();
    }
  });
}
