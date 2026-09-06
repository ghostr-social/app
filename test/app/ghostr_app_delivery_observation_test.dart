import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/ghostr_app.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('app shares the supplied delivery stream across rebuilds', (
    tester,
  ) async {
    final updates = _ObservedUpdates();
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [samplePost()]),
    );
    final app = GhostrApp(dependencies: dependencies, deliveryUpdates: updates);
    await tester.pumpWidget(app);
    await tester.pumpAndSettle();
    expect(updates.listeners, 1);
    expect(find.text('For You'), findsOneWidget);
    await tester.pumpWidget(
      GhostrApp(dependencies: dependencies, deliveryUpdates: updates),
    );
    await tester.pumpAndSettle();
    expect(updates.listeners, 1);
    await tester.pumpWidget(const SizedBox.shrink());
    await updates.controller.close();
  });
}

final class _ObservedUpdates implements VideoDeliveryUpdates {
  final controller = StreamController<VideoDeliverySnapshot>.broadcast();
  var listeners = 0;

  @override
  Stream<VideoDeliverySnapshot> watchDelivery() {
    listeners++;
    return controller.stream;
  }
}
