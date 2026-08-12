import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/release_artifact.dart';

final class StableRelease {
  factory StableRelease({
    required String versionName,
    required AndroidVersionCode versionCode,
    required DateTime publishedAt,
    required Uri releaseUri,
    required Map<AndroidAbi, ReleaseArtifact> artifacts,
  }) {
    if (versionName.isEmpty || !publishedAt.isUtc || artifacts.isEmpty) {
      throw ArgumentError('Stable release values are invalid.');
    }
    return StableRelease._(
      versionName,
      versionCode,
      publishedAt,
      releaseUri,
      Map<AndroidAbi, ReleaseArtifact>.unmodifiable(artifacts),
    );
  }

  const StableRelease._(
    this.versionName,
    this.versionCode,
    this.publishedAt,
    this.releaseUri,
    this.artifacts,
  );

  final String versionName;
  final AndroidVersionCode versionCode;
  final DateTime publishedAt;
  final Uri releaseUri;
  final Map<AndroidAbi, ReleaseArtifact> artifacts;

  ReleaseArtifact? artifactFor(AndroidAbi abi) => artifacts[abi];
}
