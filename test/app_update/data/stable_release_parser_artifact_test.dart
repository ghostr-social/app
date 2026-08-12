import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/stable_release_parser.dart';

import '../support/update_manifest_fixture.dart';

void main() {
  test('rejects malformed, duplicate, or unsupported artifacts', () {
    final invalidArtifacts = <List<Object?>>[
      [],
      [stableArtifactMap('arm64-v8a'), stableArtifactMap('arm64-v8a')],
      [stableArtifactMap('x86')],
      [stableArtifactMap('arm64-v8a', size: 0)],
      [stableArtifactMap('arm64-v8a', size: 536870913)],
      [stableArtifactMap('arm64-v8a', size: 1.5)],
      [stableArtifactMap('arm64-v8a', sha256: 'A' * 64)],
      [stableArtifactMap('arm64-v8a', sha256: 'a' * 63)],
      [stableArtifactMap('arm64-v8a')..['extra'] = true],
      ['not-an-object'],
    ];

    for (final artifacts in invalidArtifacts) {
      final manifest = stableManifestMap(artifacts: artifacts);
      expect(
        () => const StableReleaseParser().parse(stableManifestJson(manifest)),
        throwsFormatException,
      );
    }
  });
}
