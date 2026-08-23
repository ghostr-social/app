import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_page_view.dart';

void main() {
  testWidgets('presentation rebuild cannot reverse a pending page commit', (
    tester,
  ) async {
    final rebuild = ValueNotifier(0);
    final active = ValueNotifier(0);
    final changes = <int>[];
    final rosterRevision = FeedRosterRevision();
    addTearDown(rebuild.dispose);
    addTearDown(active.dispose);
    await tester.pumpWidget(
      MaterialApp(
        home: ValueListenableBuilder<int>(
          valueListenable: rebuild,
          builder: (_, __, ___) => FeedPageView(
            model: FeedPageModel(
              keys: _keys,
              activePage: active.value,
              rosterRevision: rosterRevision,
            ),
            onPageChanged: changes.add,
            itemBuilder: (_, index) => Text('Page $index'),
          ),
        ),
      ),
    );
    final page = find.byType(PageView);
    final height = tester.getSize(page).height;
    final gesture = await tester.startGesture(tester.getCenter(page));
    await gesture.moveBy(Offset(0, -height * 0.23));
    await gesture.up();
    await tester.pumpAndSettle();
    expect(changes, [1]);

    rebuild.value += 1;
    await tester.pumpAndSettle();

    expect(find.text('Page 1'), findsOneWidget);
    expect(changes, [1]);
    active.value = 1;
    rebuild.value += 1;
    await tester.pumpAndSettle();
    expect(changes, [1]);
  });

  testWidgets('authoritative roster revision reconciles a pending target', (
    tester,
  ) async {
    final rebuild = ValueNotifier(0);
    final changes = <int>[];
    var rosterRevision = FeedRosterRevision();
    addTearDown(rebuild.dispose);
    await tester.pumpWidget(
      MaterialApp(
        home: ValueListenableBuilder<int>(
          valueListenable: rebuild,
          builder: (_, __, ___) => FeedPageView(
            model: FeedPageModel(keys: _keys, rosterRevision: rosterRevision),
            onPageChanged: changes.add,
            itemBuilder: (_, index) => Text('Page $index'),
          ),
        ),
      ),
    );
    final page = find.byType(PageView);
    final height = tester.getSize(page).height;
    final gesture = await tester.startGesture(tester.getCenter(page));
    await gesture.moveBy(Offset(0, -height * 0.23));
    await gesture.up();
    await tester.pumpAndSettle();
    expect(changes, [1]);

    rosterRevision = FeedRosterRevision();
    rebuild.value += 1;
    await tester.pumpAndSettle();

    expect(find.text('Page 0'), findsOneWidget);
    expect(changes, [1, 0]);
  });
}

const _keys = [ValueKey('page-0'), ValueKey('page-1')];
