import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/inline_blurhash.dart';
import 'package:ghostr/shared/widgets/inline_blurhash_preview.dart';

void main() {
  testWidgets('paints a decorative preview and replaces its raster by value', (
    tester,
  ) async {
    await tester.pumpWidget(_app('000000'));

    expect(find.byType(CustomPaint), findsWidgets);
    expect(
      find.descendant(
        of: find.byType(InlineBlurHashPreview),
        matching: find.byType(ExcludeSemantics),
      ),
      findsOneWidget,
    );
    expect(find.bySemanticsLabel('000000'), findsNothing);
    expect(_preview(tester).descriptor.encoded, '000000');

    await tester.pumpWidget(_app('00TI:j'));

    expect(_preview(tester).descriptor.encoded, '00TI:j');
  });
}

Widget _app(String encoded) {
  return MaterialApp(
    home: InlineBlurHashPreview(descriptor: InlineBlurHash.parse(encoded)),
  );
}

InlineBlurHashPreview _preview(WidgetTester tester) {
  return tester.widget(find.byType(InlineBlurHashPreview));
}
