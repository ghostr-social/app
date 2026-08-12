import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/startup_gate.dart';

import '../support/fake_dependencies.dart';
import '../support/fake_incoming_video_share_port.dart';
import '../support/fake_video_catalog_repository.dart';

void main() {
  testWidgets('closes dependencies that finish loading after root disposal', (
    tester,
  ) async {
    final result = Completer<AppDependencies>();
    final incoming = FakeIncomingVideoSharePort();
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      device: FakeDeviceDependencies(incomingVideoShares: incoming),
    );
    await tester.pumpWidget(StartupGate(loadDependencies: () => result.future));
    await tester.pumpWidget(const SizedBox.shrink());

    result.complete(dependencies);
    await tester.pump();

    expect(incoming.closeCalls, 1);
  });
}
