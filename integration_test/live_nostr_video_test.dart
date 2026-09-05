import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/live_video_journey.dart';

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('real Nostr feed renders and advances on physical Android', (
    tester,
  ) async {
    final journey = LiveVideoJourney(binding, tester);
    try {
      await journey.run();
    } finally {
      await journey.finish();
    }
    expect(journey.failures, isEmpty, reason: journey.failures.join('\n'));
  }, timeout: const Timeout(Duration(minutes: 35)));
}
