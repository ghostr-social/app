import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_page_view.dart';

void main() {
  testWidgets('forward navigation resumes after a rapid reverse burst', (
    tester,
  ) async {
    final active = ValueNotifier(0);
    final changes = <int>[];
    addTearDown(active.dispose);
    await tester.pumpWidget(_Harness(active: active, changes: changes));

    for (var count = 0; count < 3; count++) {
      await _swipe(tester, -1);
    }
    for (var count = 0; count < 3; count++) {
      await _swipe(tester, 1);
    }
    await _swipe(tester, -1);

    expect(changes, [1, 2, 3, 2, 1, 0, 1]);
    expect(find.text('Page 1'), findsOneWidget);
  });
}

final class _Harness extends StatelessWidget {
  const _Harness({required this.active, required this.changes});

  final ValueNotifier<int> active;
  final List<int> changes;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: ValueListenableBuilder<int>(
        valueListenable: active,
        builder: (_, index, __) => FeedPageView(
          model: FeedPageModel(
            keys: _keys,
            activePage: index,
            rosterRevision: _revision,
          ),
          onPageChanged: (page) {
            changes.add(page);
            active.value = page;
          },
          itemBuilder: (_, page) => Center(child: Text('Page $page')),
        ),
      ),
    );
  }
}

Future<void> _swipe(WidgetTester tester, int direction) async {
  final page = find.byType(PageView);
  final gesture = await tester.startGesture(tester.getCenter(page));
  final distance = tester.getSize(page).height * 0.23 * direction;
  await gesture.moveBy(Offset(0, distance));
  await tester.pump(const Duration(milliseconds: 16));
  await gesture.up();
  await tester.pump(const Duration(milliseconds: 150));
}

final _revision = FeedRosterRevision();
const _keys = [
  ValueKey('page-0'),
  ValueKey('page-1'),
  ValueKey('page-2'),
  ValueKey('page-3'),
];
