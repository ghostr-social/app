import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/features/app_update/data/http_app_release_catalog.dart';
import 'package:ghostr/features/app_update/data/http_update_package_downloader.dart';
import 'package:http/testing.dart';

import '../support/app_update_cubit_harness.dart';
import '../support/fake_update_offer_history_repository.dart';

void main() {
  test('composes the HTTP updater with Android ports and settings', () async {
    final harness = AppUpdateCubitHarness();
    var disposed = false;
    final platform = AppUpdatePlatformPorts(
      installedApp: harness.installedApp,
      network: harness.network,
      installer: harness.installer,
      dispose: () async => disposed = true,
    );
    final environment = ProductionAppUpdateEnvironment(
      client: MockClient((_) async => throw StateError('not requested')),
      platform: platform,
      directoryPath: () async => '/private/updates',
    );

    final runtime = buildProductionAppUpdateRuntime(
      harness.settings,
      FakeUpdateOfferHistoryRepository(),
      environment: environment,
    );

    expect(runtime.dependencies.catalog, isA<HttpAppReleaseCatalog>());
    expect(runtime.dependencies.downloader, isA<HttpUpdatePackageDownloader>());
    expect(runtime.dependencies.installedApp, same(harness.installedApp));
    expect(runtime.dependencies.settings, same(harness.settings));
    expect(
      runtime.dependencies.offerHistory,
      isA<FakeUpdateOfferHistoryRepository>(),
    );
    await runtime.dispose();
    await runtime.dispose();
    expect(disposed, isTrue);
  });
}
