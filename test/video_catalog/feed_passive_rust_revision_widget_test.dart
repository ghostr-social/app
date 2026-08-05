import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_identity.dart';

import '../support/nostr_test_values.dart';
import '../support/rust_feed_fixtures.dart';
import '../support/rust_feed_screen_harness.dart';

void main() {
  testWidgets('main feed renders a passive Rust revision while hunting', (
    tester,
  ) async {
    final harness = await RustFeedScreenHarness.empty();

    await tester.pumpWidget(harness.app());
    await tester.pump();
    await tester.pump();
    expect(find.text('Hunting for videos'), findsOneWidget);

    harness.port.publish(
      RustFeedId.parse('1'),
      rustFeedUpdate(
        revision: 2,
        posts: [
          rustFeedPost(
            eventId: testEventId,
            caption: 'Found without another hunt',
          ),
        ],
      ),
    );
    await tester.pump();
    await tester.pump();

    expect(find.text('Found without another hunt'), findsOneWidget);
    expect(find.text('Hunting for videos'), findsNothing);
    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump();
  });
}
