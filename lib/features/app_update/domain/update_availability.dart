import 'package:ghostr/features/app_update/domain/release_artifact.dart';
import 'package:ghostr/features/app_update/domain/stable_release.dart';

sealed class AppUpdateAvailability {
  const AppUpdateAvailability();
}

final class AppUpdateCurrent extends AppUpdateAvailability {
  const AppUpdateCurrent();
}

final class AppUpdateAvailable extends AppUpdateAvailability {
  const AppUpdateAvailable({required this.release, required this.artifact});

  final StableRelease release;
  final ReleaseArtifact artifact;
}

enum AppUpdateUnsupportedReason { noCompatibleArtifact }

final class AppUpdateUnsupported extends AppUpdateAvailability {
  const AppUpdateUnsupported(this.reason);

  final AppUpdateUnsupportedReason reason;
}
