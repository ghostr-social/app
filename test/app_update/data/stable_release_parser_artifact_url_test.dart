import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/stable_release_parser.dart';

import '../support/update_manifest_fixture.dart';

void main() {
  test('accepts only the matching immutable GitHub APK asset URL', () {
    for (final url in [
      'http://github.com/ghostr-social/app/releases/download/v1.2.3/'
          'ghostr-v1.2.3-arm64-v8a.apk',
      'https://evil.example/download/v1.2.3/ghostr-v1.2.3-arm64-v8a.apk',
      'https://github.com/ghostr-social/app/releases/download/v9.9.9/'
          'ghostr-v1.2.3-arm64-v8a.apk',
      'https://github.com/ghostr-social/app/releases/download/v1.2.3/'
          'ghostr-v1.2.3-x86_64.apk',
      'https://github.com/ghostr-social/app/releases/download/v1.2.3/'
          'ghostr-v1.2.3-arm64-v8a.apk#fragment',
    ]) {
      final artifact = stableArtifactMap('arm64-v8a', url: url);
      final manifest = stableManifestMap(artifacts: [artifact]);
      expect(
        () => const StableReleaseParser().parse(stableManifestJson(manifest)),
        throwsFormatException,
        reason: url,
      );
    }
  });
}
