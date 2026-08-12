import 'package:ghostr/features/app_update/data/http_app_release_catalog.dart';
import 'package:ghostr/features/app_update/data/http_update_package_downloader.dart';
import 'package:ghostr/features/app_update/domain/installed_app_port.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/presentation/app_update_dependencies.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/platform/app_update/android_app_update_adapter.dart';
import 'package:ghostr/platform/app_update/android_app_update_platform.dart';
import 'package:http/http.dart' as http;
import 'package:path_provider/path_provider.dart';

typedef AppUpdateDisposer = Future<void> Function();

final class AppUpdateRuntime {
  const AppUpdateRuntime({
    required this.dependencies,
    required AppUpdateDisposer dispose,
  }) : _dispose = dispose;

  final AppUpdateDependencies dependencies;
  final AppUpdateDisposer _dispose;

  Future<void> dispose() => _dispose();
}

final class AppUpdatePlatformPorts {
  const AppUpdatePlatformPorts({
    required this.installedApp,
    required this.network,
    required this.installer,
    required AppUpdateDisposer dispose,
  }) : _dispose = dispose;

  factory AppUpdatePlatformPorts.android() {
    final adapter = AndroidAppUpdateAdapter(AndroidAppUpdatePlatform());
    return AppUpdatePlatformPorts(
      installedApp: adapter,
      network: adapter,
      installer: adapter,
      dispose: adapter.dispose,
    );
  }

  final InstalledAppPort installedApp;
  final NetworkConnectionPort network;
  final UpdateInstallerPort installer;
  final AppUpdateDisposer _dispose;

  Future<void> dispose() => _dispose();
}

final class ProductionAppUpdateEnvironment {
  ProductionAppUpdateEnvironment({
    required this.client,
    required this.platform,
    required this.directoryPath,
  });

  factory ProductionAppUpdateEnvironment.android() {
    return ProductionAppUpdateEnvironment(
      client: http.Client(),
      platform: AppUpdatePlatformPorts.android(),
      directoryPath: _updateDirectoryPath,
    );
  }

  final http.Client client;
  final AppUpdatePlatformPorts platform;
  final UpdateDirectoryPath directoryPath;
  var _disposed = false;

  Future<void> dispose() async {
    if (_disposed) return;
    _disposed = true;
    client.close();
    await platform.dispose();
  }
}

AppUpdateRuntime buildProductionAppUpdateRuntime(
  AppSettingsRepository settings, {
  ProductionAppUpdateEnvironment? environment,
}) {
  final edge = environment ?? ProductionAppUpdateEnvironment.android();
  return AppUpdateRuntime(
    dependencies: AppUpdateDependencies(
      catalog: HttpAppReleaseCatalog(edge.client),
      installedApp: edge.platform.installedApp,
      network: edge.platform.network,
      downloader: HttpUpdatePackageDownloader(
        client: edge.client,
        directoryPath: edge.directoryPath,
      ),
      installer: edge.platform.installer,
      settings: settings,
    ),
    dispose: edge.dispose,
  );
}

Future<String> _updateDirectoryPath() async {
  final support = await getApplicationSupportDirectory();
  return '${support.path}/updates';
}
