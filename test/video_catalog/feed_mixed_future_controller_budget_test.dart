import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';

import '../support/mixed_future_playback_fixture.dart';

void main() {
  testWidgets('counts HLS futures before retaining back history', (
    tester,
  ) async {
    final fixture = MixedFuturePlaybackFixture();
    addTearDown(fixture.close);
    await fixture.prepare(tester, ['p4', 'p5']);

    _expectCards(['p1', 'p2', 'p3', 'h0', 'h1', 'h2']);
    expect(_card('p0'), findsNothing);
    _expectReserves(['p4', 'p5']);
    _expectLiveSurfaceCount(8);
  });

  testWidgets('caps deep progressive reserves behind nearer HLS', (
    tester,
  ) async {
    final fixture = MixedFuturePlaybackFixture();
    addTearDown(fixture.close);
    await fixture.prepare(tester, ['p4', 'p5', 'p6', 'p7', 'p8']);

    _expectCards(['p3', 'h0', 'h1', 'h2']);
    _expectReserves(['p4', 'p5', 'p6', 'p7']);
    expect(_reserve('p8'), findsNothing);
    _expectLiveSurfaceCount(8);
  });
}

void _expectCards(List<String> ids) {
  for (final id in ids) {
    expect(_card(id), findsOneWidget);
  }
}

void _expectReserves(List<String> ids) {
  for (final id in ids) {
    expect(_reserve(id), findsOneWidget);
  }
}

void _expectLiveSurfaceCount(int count) {
  final cards = find.byType(FeedCard, skipOffstage: false).evaluate().length;
  expect(cards + _reserves().evaluate().length, count);
}

Finder _card(String id) => find.byWidgetPredicate(
  (widget) => widget is FeedCard && widget.post.id.value == id,
  skipOffstage: false,
);

Finder _reserve(String id) {
  return find.byKey(ValueKey('warp-reserve-$id'), skipOffstage: false);
}

Finder _reserves() => find.byWidgetPredicate(
  (widget) => switch (widget.key) {
    ValueKey<String>(value: final value) => value.startsWith('warp-reserve-'),
    _ => false,
  },
  skipOffstage: false,
);
