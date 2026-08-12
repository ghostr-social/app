import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/ghostr_app.dart';
import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/features/app_update/presentation/app_update_dependencies.dart';

import '../support/app_update_cubit_harness.dart';
import '../support/fakes.dart';

void main() {
  testWidgets('direct app teardown disposes its update runtime', (
    tester,
  ) async {
    final updates = AppUpdateCubitHarness();
    var disposeCalls = 0;
    final runtime = AppUpdateRuntime(
      dependencies: AppUpdateDependencies(
        catalog: updates.catalog,
        installedApp: updates.installedApp,
        network: updates.network,
        downloader: updates.downloader,
        installer: updates.installer,
        settings: updates.settings,
      ),
      dispose: () async => disposeCalls += 1,
    );
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      appUpdateRuntime: runtime,
    );
    await tester.pumpWidget(GhostrApp(dependencies: dependencies));
    await tester.pump();

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump();

    expect(disposeCalls, 1);
  });
}
