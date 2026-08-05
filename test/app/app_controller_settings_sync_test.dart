import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fake_dependencies.dart';
import '../support/fake_video_catalog_repository.dart';
import '../support/recording_engine_updaters.dart';

void main() {
  test('saving via the settings cubit pushes delivery config to the engine',
      () async {
    final updater = RecordingDeliveryConfigUpdater();
    final factory = AppControllerFactory(
      buildFakeDependencies(
        catalogRepository: FakeVideoCatalogRepository(forYouFeed: []),
      ),
      deliveryConfigUpdater: updater.call,
    );
    final cubit = factory.settings();
    addTearDown(cubit.close);
    await cubit.load();

    cubit.changeDataUsage(DataUsageLevel.aggressive);
    await cubit.save();

    final update = updater.pushes.single;
    expect(update.dataUsage, DataUsageLevel.aggressive);
    expect(
      update.inventoryBudget,
      AppSettings.defaults().inventoryBudget,
    );
  });
}
