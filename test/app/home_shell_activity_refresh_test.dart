import 'package:flutter_test/flutter_test.dart';

import '../support/fake_activity_repository.dart';
import '../support/fake_dependencies.dart';
import '../support/fake_video_catalog_repository.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('returning to Activity reloads new items', (tester) async {
    final activity = FakeActivityRepository();
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [samplePost()],
      ),
      device: FakeDeviceDependencies(activity: activity),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Activity'));
    await tester.pumpAndSettle();
    expect(find.text('No activity yet'), findsOneWidget);
    await tester.tap(find.text('Home'));
    await tester.pumpAndSettle();
    await activity.record(sampleActivity());
    await tester.tap(find.text('Activity'));
    await tester.pumpAndSettle();

    expect(find.text('Published a video'), findsOneWidget);
  });
}
