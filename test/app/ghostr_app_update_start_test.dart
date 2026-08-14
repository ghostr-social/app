import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/ghostr_app.dart';
import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../support/app_update_cubit_harness.dart';
import '../support/fakes.dart';

void main() {
  testWidgets('starts the updater alongside application startup', (
    tester,
  ) async {
    final updates = AppUpdateCubitHarness();
    final runtime = AppUpdateRuntime(
      dependencies: AppUpdateDependencies(
        catalog: updates.catalog,
        installedApp: updates.installedApp,
        network: updates.network,
        downloader: updates.downloader,
        installer: updates.installer,
        offerHistory: updates.offerHistory,
        settings: updates.settings,
      ),
      dispose: () async {},
    );
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      overrides: FakeDependencyOverrides(appUpdateRuntime: runtime),
    );

    await tester.pumpWidget(GhostrApp(dependencies: dependencies));
    await tester.pump();

    expect(updates.catalog.calls, 1);
    await tester.pumpWidget(const SizedBox.shrink());
  });
}
