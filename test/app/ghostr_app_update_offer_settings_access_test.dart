import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/ghostr_app.dart';
import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

import '../support/app_update_cubit_harness.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('an update offer does not cover the Settings entry', (
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
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      overrides: FakeDependencyOverrides(appUpdateRuntime: runtime),
    );

    await tester.pumpWidget(GhostrApp(dependencies: dependencies));
    await tester.pumpAndSettle();
    expect(find.text('Skip this version'), findsOneWidget);
    await tester.tap(find.byIcon(Icons.person_rounded));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Open settings'));
    await tester.pumpAndSettle();

    expect(find.text('Settings'), findsOneWidget);
    await tester.pumpWidget(const SizedBox.shrink());
  });
}
