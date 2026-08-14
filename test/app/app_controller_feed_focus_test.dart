import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';

import '../support/fake_dependencies.dart';
import '../support/fake_feed_focus_port.dart';
import '../support/fake_video_catalog_repository.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'the produced feed cubit reports focus through the injected port',
    () async {
      final focusPort = FakeFeedFocusPort();
      final factory = AppControllerFactory(
        buildFakeDependencies(
          catalogRepository: FakeVideoCatalogRepository(
            forYouFeed: [
              for (var index = 0; index < 12; index += 1)
                samplePost(id: 'post-$index'),
            ],
          ),
        ),
        feedFocus: focusPort,
      );
      final cubit = factory.feed();
      addTearDown(cubit.close);
      cubit.surfaceVisibilityChanged(true);

      await cubit.load();

      expect(focusPort.focuses.single.currentIndex, 0);
      expect(
        focusPort.focuses.single.current.media.remoteUrl,
        'https://example.com/video/post-0.mp4',
      );
    },
  );
}
