import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_kind_selector.dart';
import 'package:ghostr/shared/theme/app_theme.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

void main() {
  testWidgets('feed selector chrome stays nearly transparent over video', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: buildAppTheme(),
        home: Scaffold(
          body: FeedKindSelector(selected: FeedKind.forYou, onSelected: (_) {}),
        ),
      ),
    );

    final selectorTree = find.descendant(
      of: find.byType(FeedKindSelector),
      matching: find.byType(Material),
    );
    final materialColors = tester
        .widgetList<Material>(selectorTree)
        .map((material) => material.color)
        .whereType<Color>();
    expect(materialColors, isNotEmpty);
    expect(materialColors, everyElement(predicate<Color>((c) => c.a <= 0.1)));
    final selector = tester.widget<SegmentedButton<FeedKind>>(
      find.byType(SegmentedButton<FeedKind>),
    );
    expect(selector.style, isNotNull);
    final style = selector.style;
    if (style == null) return;
    final unselected = style.backgroundColor!.resolve({})!;
    final selected = style.backgroundColor!.resolve({WidgetState.selected})!;
    expect(unselected.a, lessThanOrEqualTo(0.1));
    expect(selected.a, lessThanOrEqualTo(0.1));
    expect(style.side!.resolve({})!.style, BorderStyle.none);
    expect(
      style.side!.resolve({WidgetState.selected})!.style,
      BorderStyle.none,
    );
    expect(_effectiveTextColor(tester, 'For You'), AppPalette.foreground);
    expect(
      _effectiveTextColor(tester, 'Following'),
      AppPalette.mutedForeground,
    );
    expect(tester.getSize(find.byType(SegmentedButton<FeedKind>)).height, 48);
  });
}

Color? _effectiveTextColor(WidgetTester tester, String label) {
  final finder = find.text(label);
  final text = tester.widget<Text>(finder);
  return DefaultTextStyle.of(
    tester.element(finder),
  ).style.merge(text.style).color;
}
