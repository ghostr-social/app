import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_hunt.dart';

void main() {
  test('a filled feed cancels the pending attempt and resets the backoff', () {
    fakeAsync((async) {
      var attempts = 0;
      final hunt = FeedHunt(
        base: const Duration(seconds: 2),
        cap: const Duration(seconds: 30),
      );

      hunt.emptied(() => attempts += 1);
      async.elapse(const Duration(seconds: 2));
      hunt.emptied(() => attempts += 1);
      hunt.filled();
      async.elapse(const Duration(minutes: 2));
      expect(attempts, 1);

      hunt.emptied(() => attempts += 1);
      async.elapse(const Duration(seconds: 2));
      expect(attempts, 2);

      hunt.emptied(() => attempts += 1);
      hunt.dispose();
      async.elapse(const Duration(minutes: 2));
      expect(attempts, 2);
    });
  });
}
