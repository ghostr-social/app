import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';

import '../support/future_hls_decoder_fixture.dart';

void main() {
  testWidgets('bounds exact future HLS decoder reserves across navigation', (
    tester,
  ) async {
    final fixture = FutureHlsDecoderFixture();
    addTearDown(fixture.close);
    await fixture.mountAndPublish(tester);
    _expectNearestPrepared(fixture);
    _expectAbsent(['h3', 'h4']);
    await fixture.promoteFourthAndReturn(tester);
    _expectAbsent(['h3', 'h4']);
    tester.binding.handleMemoryPressure();
    await tester.pumpAndSettle();
    expect(_card('h0'), findsOneWidget);
    _expectAbsent(['h1', 'h2']);
  });
}

void _expectNearestPrepared(FutureHlsDecoderFixture fixture) {
  for (final id in ['h0', 'h1', 'h2']) {
    final request = fixture.request(id);
    expect(request.isActive, isFalse);
    expect(request.hlsAuthority, fixture.authority(id));
    expect(request.reservesPreparedDecoder, isTrue);
    expect(request.keepWarmWhenInactive, isTrue);
    fixture.render(id);
  }
}

void _expectAbsent(List<String> ids) {
  for (final id in ids) {
    expect(_card(id), findsNothing);
  }
}

Finder _card(String id) => find.byWidgetPredicate(
  (widget) => widget is FeedCard && widget.post.id.value == id,
  skipOffstage: false,
);
