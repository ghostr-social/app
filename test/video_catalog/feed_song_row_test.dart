import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('shows the song name next to a music note icon', (tester) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost()],
    );

    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();

    expect(find.text('Original sound'), findsOneWidget);
    expect(find.byIcon(Icons.music_note), findsOneWidget);
  });
}
