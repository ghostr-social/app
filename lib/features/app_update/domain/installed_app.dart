import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';

final class InstalledApp {
  factory InstalledApp({
    required String packageName,
    required String versionName,
    required AndroidVersionCode versionCode,
    required List<AndroidAbi> supportedAbis,
  }) {
    if (packageName.isEmpty || versionName.isEmpty) {
      throw ArgumentError('Installed app identity is required.');
    }
    return InstalledApp._(
      packageName,
      versionName,
      versionCode,
      List<AndroidAbi>.unmodifiable(supportedAbis),
    );
  }

  const InstalledApp._(
    this.packageName,
    this.versionName,
    this.versionCode,
    this.supportedAbis,
  );

  final String packageName;
  final String versionName;
  final AndroidVersionCode versionCode;
  final List<AndroidAbi> supportedAbis;
}
