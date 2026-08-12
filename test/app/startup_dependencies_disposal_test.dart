import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/startup_gate.dart';

import '../support/fake_dependencies.dart';
import '../support/fake_incoming_video_share_port.dart';
import '../support/fake_video_catalog_repository.dart';

void main() {
  testWidgets('closes loaded app dependencies when the root is removed', (
    tester,
  ) async {
    final incoming = FakeIncomingVideoSharePort();
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      device: FakeDeviceDependencies(incomingVideoShares: incoming),
    );
    await tester.pumpWidget(
      StartupGate(loadDependencies: () async => dependencies),
    );
    await tester.pumpAndSettle();

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump();

    expect(incoming.closeCalls, 1);
  });
}
