import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/update_package_sha256.dart';

final class VerifiedUpdatePackage {
  factory VerifiedUpdatePackage({
    required String path,
    required AndroidVersionCode versionCode,
    required AndroidAbi abi,
    required int sizeBytes,
    required UpdatePackageSha256 sha256,
  }) {
    if (path.isEmpty || sizeBytes < 1) {
      throw ArgumentError('Verified package values are invalid.');
    }
    return VerifiedUpdatePackage._(path, versionCode, abi, sizeBytes, sha256);
  }

  const VerifiedUpdatePackage._(
    this.path,
    this.versionCode,
    this.abi,
    this.sizeBytes,
    this.sha256,
  );

  final String path;
  final AndroidVersionCode versionCode;
  final AndroidAbi abi;
  final int sizeBytes;
  final UpdatePackageSha256 sha256;
}
