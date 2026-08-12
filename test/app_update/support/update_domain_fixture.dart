import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/release_artifact.dart';
import 'package:ghostr/features/app_update/domain/stable_release.dart';
import 'package:ghostr/features/app_update/domain/update_package_sha256.dart';

StableRelease sampleStableRelease({
  int versionCode = 2,
  List<AndroidAbi> abis = AndroidAbi.values,
}) {
  const versionName = '0.0.2';
  return StableRelease(
    versionName: versionName,
    versionCode: AndroidVersionCode(versionCode),
    publishedAt: DateTime.utc(2026, 8, 11, 12),
    releaseUri: Uri.parse(
      'https://github.com/ghostr-social/app/releases/tag/v$versionName',
    ),
    artifacts: {for (final abi in abis) abi: sampleArtifact(abi)},
  );
}

ReleaseArtifact sampleArtifact(
  AndroidAbi abi, {
  int sizeBytes = 4,
  String? digest,
}) {
  return ReleaseArtifact(
    abi: abi,
    uri: Uri.parse(
      'https://github.com/ghostr-social/app/releases/download/'
      'v0.0.2/ghostr-v0.0.2-${abi.value}.apk',
    ),
    sizeBytes: sizeBytes,
    sha256: UpdatePackageSha256.parse(digest ?? 'a' * 64),
  );
}
