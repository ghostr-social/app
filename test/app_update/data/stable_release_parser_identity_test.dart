import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/stable_release_parser.dart';

import '../support/update_manifest_fixture.dart';

void main() {
  test('rejects metadata outside the stable Ghostr schema identity', () {
    final valid = stableManifestMap();
    final invalidValues = <String, Object?>{
      'schemaVersion': 2,
      'namespace': 'example.social',
      'packageName': 'social.ghostr',
      'channel': 'beta',
    };

    for (final entry in invalidValues.entries) {
      final json = stableManifestJson({...valid, entry.key: entry.value});
      expect(
        () => const StableReleaseParser().parse(json),
        throwsFormatException,
        reason: entry.key,
      );
    }
  });

  test('rejects unknown and missing top-level fields', () {
    final unknown = {...stableManifestMap(), 'unexpected': true};
    final missing = {...stableManifestMap()}..remove('versionCode');

    expect(
      () => const StableReleaseParser().parse(stableManifestJson(unknown)),
      throwsFormatException,
    );
    expect(
      () => const StableReleaseParser().parse(stableManifestJson(missing)),
      throwsFormatException,
    );
    expect(
      () => const StableReleaseParser().parse('{not-json'),
      throwsFormatException,
    );
    expect(
      () => const StableReleaseParser().parse('[]'),
      throwsFormatException,
    );
  });
}
