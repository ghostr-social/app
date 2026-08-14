import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_repost_action.dart';

import '../support/repost_samples.dart';

void main() {
  testWidgets('unavailable repost action is disabled and explained', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: FeedCardRepostAction(post: repostablePost(), onToggle: null),
        ),
      ),
    );

    const tooltip = 'Reposting unavailable for this video';
    final button = tester.widget<IconButton>(find.byType(IconButton));
    expect(button.onPressed, isNull);
    expect(
      tester.getSemantics(find.byTooltip(tooltip)),
      isSemantics(
        tooltip: tooltip,
        hasSelectedState: true,
        isSelected: false,
        isButton: true,
        isEnabled: false,
        hasTapAction: false,
      ),
    );
    semantics.dispose();
  });
}
