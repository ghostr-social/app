import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/stable_release_parser.dart';

import '../support/update_manifest_fixture.dart';

void main() {
  test('accepts only real UTC timestamps with whole-second precision', () {
    for (final timestamp in [
      '2026-08-11 12:00:00Z',
      '2026-08-11T12:00:00-03:00',
      '2026-08-11T12:00:00.000Z',
      '2026-02-30T12:00:00Z',
      '2026-99-11T12:00:00Z',
      'not-a-time',
    ]) {
      final manifest = stableManifestMap(publishedAt: timestamp);
      expect(
        () => const StableReleaseParser().parse(stableManifestJson(manifest)),
        throwsFormatException,
        reason: timestamp,
      );
    }
  });
}
