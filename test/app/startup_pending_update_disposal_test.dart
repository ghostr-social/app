import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/app/startup_gate.dart';
import 'package:ghostr/features/app_update/presentation/app_update_dependencies.dart';

import '../support/app_update_cubit_harness.dart';
import '../support/fake_dependencies.dart';
import '../support/fake_incoming_video_share_port.dart';
import '../support/fake_video_catalog_repository.dart';

void main() {
  testWidgets('closes a late-loaded update runtime after root disposal', (
    tester,
  ) async {
    final result = Completer<AppDependencies>();
    final incoming = FakeIncomingVideoSharePort();
    var updateCloseCalls = 0;
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      device: FakeDeviceDependencies(incomingVideoShares: incoming),
      appUpdateRuntime: _runtime(() async => updateCloseCalls += 1),
    );
    await tester.pumpWidget(StartupGate(loadDependencies: () => result.future));
    await tester.pumpWidget(const SizedBox.shrink());

    result.complete(dependencies);
    await tester.pump();

    expect(incoming.closeCalls, 1);
    expect(updateCloseCalls, 1);
  });
}

AppUpdateRuntime _runtime(AppUpdateDisposer dispose) {
  final updates = AppUpdateCubitHarness();
  return AppUpdateRuntime(
    dependencies: AppUpdateDependencies(
      catalog: updates.catalog,
      installedApp: updates.installedApp,
      network: updates.network,
      downloader: updates.downloader,
      installer: updates.installer,
      offerHistory: updates.offerHistory,
      settings: updates.settings,
    ),
    dispose: dispose,
  );
}
