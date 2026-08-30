import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('spare future capacity keeps the bounded back stack warm', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 7);
    addTearDown(fixture.updates.close);
    await fixture.pump(tester);
    fixture.publishWindow(1, 'p0', _initialFuture);
    await fixture.settle(tester);
    final retained = {
      for (final id in ['p1', 'p2', 'p3'])
        id: fixture.platform.playerFor(fixture.url(id)),
    };
    await _moveToP4(tester, fixture);
    final element = tester.element(find.byType(PageView));
    _expectBoundedRoster(element);
    _expectRetained(fixture, retained);
    await _reuseBackStack(tester, fixture, element.read<FeedCubit>(), retained);
    expect(fixture.platform.peakPlayerCount, lessThanOrEqualTo(8));
    expect(fixture.platform.audibleOverlap, isFalse);
  });
}

Future<void> _moveToP4(
  WidgetTester tester,
  FeedPreparationFixture fixture,
) async {
  for (var index = 0; index < 4; index += 1) {
    await fixture.swipe(tester);
  }
  fixture.publishWindow(2, 'p4', ['p5', 'p6']);
  await fixture.settle(tester);
}

void _expectBoundedRoster(Element element) {
  final state = element.read<FeedCubit>().state as FeedLoaded;
  expect(state.posts.map((post) => post.id.value), _boundedRoster);
  expect(state.activeIndex, 3);
}

void _expectRetained(FeedPreparationFixture fixture, Map<String, int> players) {
  for (final player in players.values) {
    expect(fixture.platform.disposed, isNot(contains(player)));
    expect(_texture(player), findsOneWidget);
  }
}

Future<void> _reuseBackStack(
  WidgetTester tester,
  FeedPreparationFixture fixture,
  FeedCubit cubit,
  Map<String, int> players,
) async {
  for (final target in _backStack) {
    cubit.pageChanged(target.index);
    await fixture.settle(tester);
    final player = players[target.id]!;
    expect(fixture.platform.playerFor(fixture.url(target.id)), player);
    expect(fixture.platform.creationsFor(fixture.url(target.id)), 1);
    expect(find.text('Caption ${target.id}'), findsOneWidget);
  }
}

Finder _texture(int player) {
  return find.byWidgetPredicate(
    (widget) => widget is Texture && widget.textureId == player,
    skipOffstage: false,
  );
}

const _initialFuture = ['p1', 'p2', 'p3', 'p4', 'p5', 'p6'];
const _boundedRoster = ['p1', 'p2', 'p3', 'p4', 'p5', 'p6'];
const _backStack = [
  (index: 2, id: 'p3'),
  (index: 1, id: 'p2'),
  (index: 0, id: 'p1'),
];
