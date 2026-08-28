import 'package:flutter_test/flutter_test.dart';

import '../support/production_hls_first_frame_fixture.dart';

void main() {
  testWidgets(
    'production HLS reports its exact frame without ranking feedback',
    (tester) async {
      final fixture = ProductionHlsFirstFrameFixture();
      addTearDown(fixture.close);
      await fixture.mount(tester);

      expect(fixture.deliveryId, 'hls-post');
      expect(fixture.token, isNotNull);
      expect(fixture.feedback.events, isEmpty);
      expect(fixture.observationPosts, isNotEmpty);
      expect(fixture.observationPosts, everyElement('hls-post'));

      fixture.emitNativeFrame();
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      expect(fixture.presentationPosts, isEmpty);
      await fixture.settlePresentation(tester);
      expect(fixture.presentationPosts, ['hls-post']);

      fixture.emitNativeFrame();
      await tester.pump();
      expect(fixture.presentationPosts, hasLength(1));
    },
  );
}
