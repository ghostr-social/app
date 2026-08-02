import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

void main() {
  testWidgets('announces the workflow represented by a loading spinner',
      (tester) async {
    await tester.pumpWidget(const MaterialApp(
      home: LoadingPanel(label: 'Loading activity'),
    ));

    expect(find.bySemanticsLabel('Loading activity'), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
  });
}
