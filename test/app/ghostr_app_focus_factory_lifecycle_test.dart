import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/ghostr_app.dart';
import 'package:ghostr/app/session_gate.dart';

import '../support/fakes.dart';
import '../support/fake_feed_focus_port.dart';

void main() {
  testWidgets(
    'app rebuild preserves the focus arbiter and FFI generation owner',
    (tester) async {
      final dependencies = buildFakeDependencies(
        catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      );
      final focus = FakeFeedFocusPort();
      await tester.pumpWidget(
        GhostrApp(dependencies: dependencies, feedFocus: focus),
      );
      final first = tester.widget<SessionGate>(find.byType(SessionGate));

      await tester.pumpWidget(
        GhostrApp(dependencies: dependencies, feedFocus: focus),
      );
      final rebuilt = tester.widget<SessionGate>(find.byType(SessionGate));

      expect(rebuilt.controllers, same(first.controllers));
    },
  );
}
