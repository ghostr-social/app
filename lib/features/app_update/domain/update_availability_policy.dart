import 'package:ghostr/features/app_update/domain/installed_app.dart';
import 'package:ghostr/features/app_update/domain/stable_release.dart';
import 'package:ghostr/features/app_update/domain/update_availability.dart';

final class UpdateAvailabilityPolicy {
  const UpdateAvailabilityPolicy();

  AppUpdateAvailability evaluate({
    required InstalledApp installed,
    required StableRelease release,
  }) {
    if (release.versionCode.compareTo(installed.versionCode) <= 0) {
      return const AppUpdateCurrent();
    }
    for (final abi in installed.supportedAbis) {
      final artifact = release.artifactFor(abi);
      if (artifact != null) {
        return AppUpdateAvailable(release: release, artifact: artifact);
      }
    }
    return const AppUpdateUnsupported(
      AppUpdateUnsupportedReason.noCompatibleArtifact,
    );
  }
}
