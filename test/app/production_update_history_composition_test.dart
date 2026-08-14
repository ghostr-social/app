import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/features/app_update/data/local_update_offer_history_repository.dart';
import 'package:ghostr/features/app_update/domain/update_offer_history_repository.dart';
import 'package:ghostr/features/app_update/presentation/app_update_dependencies.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/app_update_cubit_harness.dart';
import '../support/fake_nostr_event_client.dart';
import '../support/fake_nostr_session_port.dart';
import '../support/fake_nostr_social_port.dart';
import '../support/fake_nostr_video_publisher_port.dart';
import '../support/fake_remote_video_source.dart';
import '../support/nostr_test_values.dart';
import '../support/test_video_delivery.dart';

void main() {
  test('production graph persists update offer history locally', () async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    final preferences = await SharedPreferences.getInstance();
    final harness = AppUpdateCubitHarness();
    UpdateOfferHistoryRepository? capturedHistory;
    final nostr = ProductionNostrServices(
      ProductionNostrAdapters(FakeNostrSessionPort(), FakeNostrSocialPort()),
      FakeNostrEventClient(publicKeyHex: testViewerPublicKey),
      FakeNostrVideoPublisherPort(),
    );
    final environment = ProductionDependenciesEnvironment(
      preferencesLoader: () async => preferences,
      nostrServicesBuilder: (_) => nostr,
      videoDeliveryBuilder: (_, __) async =>
          testVideoDelivery(remoteSource: FakeRemoteVideoSource([])),
      appUpdateBuilder: (settings, history) {
        capturedHistory = history;
        return AppUpdateRuntime(
          dependencies: AppUpdateDependencies(
            catalog: harness.catalog,
            installedApp: harness.installedApp,
            network: harness.network,
            downloader: harness.downloader,
            installer: harness.installer,
            offerHistory: history,
            settings: settings,
          ),
          dispose: () async {},
        );
      },
    );

    final dependencies = await buildProductionDependencies(environment);

    expect(capturedHistory, isA<LocalUpdateOfferHistoryRepository>());
    expect(
      dependencies.appUpdateRuntime!.dependencies.offerHistory,
      same(capturedHistory),
    );
    await dependencies.appUpdateRuntime!.dispose();
  });
}
