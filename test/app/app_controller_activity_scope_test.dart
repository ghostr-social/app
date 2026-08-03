import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';

import '../support/fake_activity_repository.dart';
import '../support/fake_dependencies.dart';
import '../support/fake_video_catalog_repository.dart';

void main() {
  test('pins activity storage to the account owning the controller', () {
    final activity = FakeActivityRepository();
    final factory = AppControllerFactory(buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      device: FakeDeviceDependencies(activity: activity),
    ));

    final cubit = factory.activity();
    addTearDown(cubit.close);

    expect(activity.activeAccountSnapshots, 1);
  });
}
