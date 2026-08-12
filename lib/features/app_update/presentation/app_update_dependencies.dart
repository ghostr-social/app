import 'package:ghostr/features/app_update/domain/app_release_catalog.dart';
import 'package:ghostr/features/app_update/domain/installed_app_port.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/domain/update_installer_port.dart';
import 'package:ghostr/features/app_update/domain/update_package_downloader.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';

final class AppUpdateDependencies {
  const AppUpdateDependencies({
    required this.catalog,
    required this.installedApp,
    required this.network,
    required this.downloader,
    required this.installer,
    required this.settings,
  });

  final AppReleaseCatalog catalog;
  final InstalledAppPort installedApp;
  final NetworkConnectionPort network;
  final UpdatePackageDownloader downloader;
  final UpdateInstallerPort installer;
  final AppSettingsRepository settings;
}
