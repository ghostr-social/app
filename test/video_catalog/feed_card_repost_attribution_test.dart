import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_metadata.dart';

import '../support/repost_samples.dart';

void main() {
  testWidgets('shows reposter attribution above the original creator', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: FeedCardMetadata(post: repostedPost(), onOpenHashtag: (_) {}),
        ),
      ),
    );

    expect(find.text('Bob Relay reposted'), findsOneWidget);
    expect(find.text('Nora Relay'), findsOneWidget);
    expect(
      find.bySemanticsLabel(RegExp('reposted'), skipOffstage: false),
      findsOneWidget,
    );
    expect(
      find.bySemanticsLabel('Bob Relay reposted this video'),
      findsOneWidget,
    );
    semantics.dispose();
  });
}
