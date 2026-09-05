import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/live_comparison_surface.dart';

void main() {
  testWidgets(
    'direct controls identify the phone test visibly and semantically',
    (tester) async {
      final semantics = tester.ensureSemantics();
      await tester.pumpWidget(
        const LiveComparisonSurface(
          label: 'Direct playback',
          host: 'media.example',
          child: SizedBox(),
        ),
      );
      expect(find.text('Phone video test — Direct playback'), findsOneWidget);
      expect(
        find.bySemanticsLabel('Phone video test — Direct playback'),
        findsOneWidget,
      );
      expect(find.text('media.example'), findsOneWidget);
      semantics.dispose();
    },
  );
}
