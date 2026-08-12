import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/data/stable_release_parser.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';

import '../support/update_manifest_fixture.dart';

void main() {
  test('parses trusted stable metadata into immutable domain values', () {
    final release = const StableReleaseParser().parse(stableManifestJson());

    expect(release.versionName, '1.2.3');
    expect(release.versionCode.value, 1002003);
    expect(release.publishedAt, DateTime.utc(2026, 8, 11, 12));
    expect(
      release.releaseUri.toString(),
      'https://github.com/ghostr-social/app/releases/tag/v1.2.3',
    );
    expect(release.artifacts.keys, AndroidAbi.values);
    expect(release.artifactFor(AndroidAbi.arm64V8a)?.sizeBytes, 4);
    expect(
      () => release.artifacts.remove(AndroidAbi.arm64V8a),
      throwsUnsupportedError,
    );
  });
}
