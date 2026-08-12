import 'package:ghostr/features/app_update/domain/release_artifact.dart';
import 'package:ghostr/features/app_update/domain/stable_release.dart';
import 'package:ghostr/features/app_update/domain/verified_update_package.dart';

abstract interface class UpdatePackageDownloader {
  Stream<UpdateDownloadEvent> download(
    StableRelease release,
    ReleaseArtifact artifact,
  );
}

sealed class UpdateDownloadEvent {
  const UpdateDownloadEvent();
}

final class UpdateDownloadProgress extends UpdateDownloadEvent {
  const UpdateDownloadProgress({required this.bytes, required this.totalBytes});

  final int bytes;
  final int totalBytes;

  double get fraction => bytes / totalBytes;
}

final class UpdateDownloadCompleted extends UpdateDownloadEvent {
  const UpdateDownloadCompleted(this.package);

  final VerifiedUpdatePackage package;
}
