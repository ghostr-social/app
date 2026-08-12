import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/update_package_sha256.dart';

final class ReleaseArtifact {
  static const maximumSizeBytes = 512 * 1024 * 1024;

  factory ReleaseArtifact({
    required AndroidAbi abi,
    required Uri uri,
    required int sizeBytes,
    required UpdatePackageSha256 sha256,
  }) {
    if (uri.scheme != 'https' || uri.host.isEmpty) {
      throw ArgumentError.value(uri, 'uri', 'HTTPS is required.');
    }
    if (sizeBytes < 1 || sizeBytes > maximumSizeBytes) {
      throw ArgumentError.value(sizeBytes, 'sizeBytes', 'Outside safe range.');
    }
    return ReleaseArtifact._(abi, uri, sizeBytes, sha256);
  }

  const ReleaseArtifact._(this.abi, this.uri, this.sizeBytes, this.sha256);

  final AndroidAbi abi;
  final Uri uri;
  final int sizeBytes;
  final UpdatePackageSha256 sha256;
}
