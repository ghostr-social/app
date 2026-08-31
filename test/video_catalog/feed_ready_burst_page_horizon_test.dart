import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('three prepared swipe targets own decoders before a burst', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 7);
    addTearDown(fixture.updates.close);
    await fixture.pump(tester);

    fixture.publishWindow(1, 'p0', ['p1', 'p2', 'p3', 'p4', 'p5', 'p6']);
    await fixture.settle(tester);

    for (final id in ['p1', 'p2', 'p3']) {
      _expectPageDecoder(tester, id);
    }
    final fourth = fixture.platform.playerFor(fixture.url('p4'));
    expect(find.byKey(const ValueKey('warp-reserve-p4')), findsOneWidget);

    await fixture.swipe(tester);

    _expectPageDecoder(tester, 'p4');
    expect(fixture.platform.creationsFor(fixture.url('p4')), 1);
    expect(fixture.platform.disposed, isNot(contains(fourth)));
    expect(fixture.platform.peakPlayerCount, lessThanOrEqualTo(8));
  });
}

void _expectPageDecoder(WidgetTester tester, String id) {
  final page = find.byWidgetPredicate(
    (widget) => widget is FeedCard && widget.post.id.value == id,
    skipOffstage: false,
  );
  expect(page, findsOneWidget, reason: '$id must be swipe-ready');
  expect(
    find.descendant(
      of: page,
      matching: find.byType(Texture, skipOffstage: false),
      skipOffstage: false,
    ),
    findsOneWidget,
    reason: '$id must retain its prepared decoder on its feed page',
  );
  expect(
    _hasKeepAliveParentData(tester, page),
    isTrue,
    reason: '$id must keep its prepared page alive between layouts',
  );
}

bool _hasKeepAliveParentData(WidgetTester tester, Finder page) {
  var keptAlive = false;
  tester.element(page).visitAncestorElements((element) {
    if (element is! RenderObjectElement) return true;
    final parentData = element.renderObject.parentData;
    if (parentData is! SliverMultiBoxAdaptorParentData) return true;
    keptAlive = parentData.keepAlive;
    return false;
  });
  return keptAlive;
}
