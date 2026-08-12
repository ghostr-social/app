import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/installed_app.dart';
import 'package:ghostr/features/app_update/domain/release_artifact.dart';
import 'package:ghostr/features/app_update/domain/stable_release.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/domain/update_package_downloader.dart';
import 'package:ghostr/features/app_update/domain/update_package_sha256.dart';
import 'package:ghostr/features/app_update/domain/verified_update_package.dart';

void main() {
  test('rejects invalid constructed updater models and exposes progress', () {
    final digest = UpdatePackageSha256.parse('a' * 64);
    expect(
      () => ReleaseArtifact(
        abi: AndroidAbi.arm64V8a,
        uri: Uri.parse('http://example.com/app.apk'),
        sizeBytes: 1,
        sha256: digest,
      ),
      throwsArgumentError,
    );
    expect(
      () => ReleaseArtifact(
        abi: AndroidAbi.arm64V8a,
        uri: Uri.parse('https://example.com/app.apk'),
        sizeBytes: 0,
        sha256: digest,
      ),
      throwsArgumentError,
    );
    expect(
      () => ReleaseArtifact(
        abi: AndroidAbi.arm64V8a,
        uri: Uri.parse('https://example.com/app.apk'),
        sizeBytes: ReleaseArtifact.maximumSizeBytes + 1,
        sha256: digest,
      ),
      throwsArgumentError,
    );
    expect(
      () => InstalledApp(
        packageName: '',
        versionName: '1',
        versionCode: AndroidVersionCode(1),
        supportedAbis: const [],
      ),
      throwsArgumentError,
    );
    expect(
      () => VerifiedUpdatePackage(
        path: '',
        versionCode: AndroidVersionCode(1),
        abi: AndroidAbi.arm64V8a,
        sizeBytes: 1,
        sha256: digest,
      ),
      throwsArgumentError,
    );
    expect(_invalidRelease, throwsArgumentError);
    final package = VerifiedUpdatePackage(
      path: '/updates/app.apk',
      versionCode: AndroidVersionCode(2),
      abi: AndroidAbi.arm64V8a,
      sizeBytes: 1,
      sha256: digest,
    );
    final request = UpdateInstallRequest(
      package: package,
      mode: UpdateInstallMode.automaticWhenPermitted,
    );
    expect(request.package, same(package));
    expect(request.mode, UpdateInstallMode.automaticWhenPermitted);
    expect(UpdateInstallSession(0).id, 0);
    expect(() => UpdateInstallSession(-1), throwsArgumentError);
    expect(
      const UpdateDownloadProgress(bytes: 1, totalBytes: 4).fraction,
      0.25,
    );
  });
}

void _invalidRelease() {
  StableRelease(
    versionName: '',
    versionCode: AndroidVersionCode(1),
    publishedAt: DateTime.utc(2026),
    releaseUri: Uri.parse('https://example.com'),
    artifacts: const {},
  );
}
