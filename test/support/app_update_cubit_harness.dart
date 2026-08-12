import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/installed_app.dart';
import 'package:ghostr/features/app_update/domain/network_connection_port.dart';
import 'package:ghostr/features/app_update/domain/stable_release.dart';
import 'package:ghostr/features/app_update/domain/update_package_downloader.dart';
import 'package:ghostr/features/app_update/domain/verified_update_package.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/core/time/clock.dart';

import '../app_update/support/update_domain_fixture.dart';
import 'app_update_installer_fake.dart';
import 'app_update_port_fakes.dart';
import 'fake_app_settings_repository.dart';

final class AppUpdateCubitHarness {
  AppUpdateCubitHarness({
    AppUpdatePreferences preferences = AppUpdatePreferences.defaults,
    InstalledApp? installed,
    StableRelease? release,
    NetworkConnection connection = NetworkConnection.wifi,
  }) : catalog = FakeAppReleaseCatalog(release ?? sampleStableRelease()),
       installedApp = FakeInstalledAppPort(installed ?? sampleInstalledApp()),
       network = FakeNetworkConnectionPort(connection),
       settings = FakeAppSettingsRepository(_settingsWith(preferences)) {
    final artifact = catalog.release.artifacts.values.first;
    package = VerifiedUpdatePackage(
      path: '/tmp/ghostr.apk',
      versionCode: catalog.release.versionCode,
      abi: artifact.abi,
      sizeBytes: artifact.sizeBytes,
      sha256: artifact.sha256,
    );
    downloader = FakeUpdatePackageDownloader([
      UpdateDownloadCompleted(package),
    ]);
  }

  final FakeAppReleaseCatalog catalog;
  final FakeInstalledAppPort installedApp;
  final FakeNetworkConnectionPort network;
  final FakeAppSettingsRepository settings;
  final installer = FakeUpdateInstallerPort();
  late final VerifiedUpdatePackage package;
  late final FakeUpdatePackageDownloader downloader;

  AppUpdateCubit build({Clock clock = systemClock}) => AppUpdateCubit(
    AppUpdateDependencies(
      catalog: catalog,
      installedApp: installedApp,
      network: network,
      downloader: downloader,
      installer: installer,
      settings: settings,
    ),
    clock: clock,
  );
}

InstalledApp sampleInstalledApp({List<AndroidAbi>? abis}) {
  return InstalledApp(
    packageName: 'app.ghostr',
    versionName: '0.0.1',
    versionCode: AndroidVersionCode(1),
    supportedAbis: abis ?? const [AndroidAbi.arm64V8a],
  );
}

AppSettings _settingsWith(AppUpdatePreferences preferences) {
  return AppSettings.defaults().copyWith(updatePreferences: preferences);
}
