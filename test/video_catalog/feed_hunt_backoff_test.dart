import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_hunt.dart';

void main() {
  test('attempts fire on a doubling delay capped at the maximum', () {
    fakeAsync((async) {
      var attempts = 0;
      final hunt = FeedHunt(
        base: const Duration(seconds: 2),
        cap: const Duration(seconds: 30),
      );

      hunt.emptied(() => attempts += 1);
      async.elapse(const Duration(seconds: 1));
      expect(attempts, 0);
      async.elapse(const Duration(seconds: 1));
      expect(attempts, 1);

      hunt.emptied(() => attempts += 1);
      async.elapse(const Duration(seconds: 2));
      expect(attempts, 1);
      async.elapse(const Duration(seconds: 2));
      expect(attempts, 2);

      for (var round = 0; round < 10; round += 1) {
        hunt.emptied(() => attempts += 1);
        async.elapse(const Duration(seconds: 30));
      }
      expect(attempts, 12);

      hunt.emptied(() => attempts += 1);
      async.elapse(const Duration(seconds: 29));
      expect(attempts, 12);
      async.elapse(const Duration(seconds: 1));
      expect(attempts, 13);
      hunt.dispose();
    });
  });
}
