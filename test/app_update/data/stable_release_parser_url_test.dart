import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/stable_release_parser.dart';

import '../support/update_manifest_fixture.dart';

void main() {
  test('rejects release pages outside the matching GitHub tag', () {
    for (final url in [
      'http://github.com/ghostr-social/app/releases/tag/v1.2.3',
      'https://evil.example/ghostr-social/app/releases/tag/v1.2.3',
      'https://github.com/other/app/releases/tag/v1.2.3',
      'https://github.com/ghostr-social/app/releases/tag/v9.9.9',
      'https://user@github.com/ghostr-social/app/releases/tag/v1.2.3',
      'https://github.com/ghostr-social/app/releases/tag/v1.2.3?q=1',
    ]) {
      final manifest = stableManifestMap(releaseUrl: url);
      expect(
        () => const StableReleaseParser().parse(stableManifestJson(manifest)),
        throwsFormatException,
        reason: url,
      );
    }
  });
}
