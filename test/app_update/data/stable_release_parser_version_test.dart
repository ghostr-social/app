import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/stable_release_parser.dart';

import '../support/update_manifest_fixture.dart';

void main() {
  test('rejects malformed version names and Android version codes', () {
    final invalid = <Map<String, Object?>>[
      stableManifestMap(versionName: 'v1.2.3'),
      stableManifestMap(versionName: '1.2'),
      {...stableManifestMap(), 'versionName': 123},
      stableManifestMap(versionCode: 0),
      stableManifestMap(versionCode: 2100000001),
      {...stableManifestMap(), 'versionCode': 1.5},
    ];

    for (final manifest in invalid) {
      expect(
        () => const StableReleaseParser().parse(stableManifestJson(manifest)),
        throwsFormatException,
      );
    }
  });
}
