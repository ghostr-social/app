import 'package:ghostr/features/app_update/domain/app_release_catalog.dart';
import 'package:ghostr/features/app_update/domain/installed_app.dart';
import 'package:ghostr/features/app_update/domain/installed_app_port.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/domain/release_artifact.dart';
import 'package:ghostr/features/app_update/domain/stable_release.dart';
import 'package:ghostr/features/app_update/domain/update_package_downloader.dart';

final class FakeAppReleaseCatalog implements AppReleaseCatalog {
  FakeAppReleaseCatalog(this.release);

  StableRelease release;
  Future<void>? beforeResult;
  Object? failure;
  int calls = 0;

  @override
  Future<StableRelease> fetchStableRelease() async {
    calls += 1;
    await beforeResult;
    if (failure != null) throw failure!;
    return release;
  }
}

final class FakeInstalledAppPort implements InstalledAppPort {
  FakeInstalledAppPort(this.installed);

  InstalledApp installed;
  Object? failure;
  int calls = 0;

  @override
  Future<InstalledApp> readInstalledApp() async {
    calls += 1;
    if (failure != null) throw failure!;
    return installed;
  }
}

final class FakeNetworkConnectionPort implements NetworkConnectionPort {
  FakeNetworkConnectionPort(this.connection);

  NetworkConnection connection;
  Object? failure;
  int calls = 0;

  @override
  Future<NetworkConnection> readConnection() async {
    calls += 1;
    if (failure != null) throw failure!;
    return connection;
  }
}

final class FakeUpdatePackageDownloader implements UpdatePackageDownloader {
  FakeUpdatePackageDownloader(this.events);

  List<UpdateDownloadEvent> events;
  Future<void>? beforeEvents;
  Object? failure;
  int calls = 0;

  @override
  Stream<UpdateDownloadEvent> download(
    StableRelease release,
    ReleaseArtifact artifact,
  ) async* {
    calls += 1;
    if (failure != null) throw failure!;
    await beforeEvents;
    for (final event in events) {
      yield event;
    }
  }
}
