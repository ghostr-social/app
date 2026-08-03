import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/media_picker_port.dart';
import 'package:ghostr/core/media/media_picker_capabilities.dart';
import 'package:ghostr/core/media/selected_media.dart';

import '../support/compose_screen_harness.dart';
import '../support/fakes.dart';

void main() {
  testWidgets('disables every composer input while selecting media',
      (tester) async {
    final picker = _PendingRecoveryPicker();
    await tester.pumpWidget(composeScreenHarness(
      publishing: FakeVideoCatalogRepository(forYouFeed: []),
      activity: FakeActivityRepository(),
      picker: picker,
    ));
    await tester.pump();

    expect(_elevated(tester, 'Choose from library').onPressed, isNull);
    expect(_filled(tester, 'Capture video').onPressed, isNull);
    expect(tester.widget<TextField>(find.byType(TextField)).enabled, isFalse);
    expect(_elevated(tester, 'Publish').onPressed, isNull);

    picker.pending.complete(null);
    await tester.pumpAndSettle();
  });
}

ElevatedButton _elevated(WidgetTester tester, String label) {
  return tester.widget(find.widgetWithText(ElevatedButton, label));
}

FilledButton _filled(WidgetTester tester, String label) {
  return tester.widget(find.widgetWithText(FilledButton, label));
}

class _PendingRecoveryPicker implements MediaPickerPort {
  final pending = Completer<SelectedMedia?>();

  @override
  MediaPickerCapabilities get capabilities =>
      const MediaPickerCapabilities.allSupported();

  @override
  Future<SelectedMedia?> recoverLostVideo() => pending.future;

  @override
  Future<SelectedMedia?> captureVideo() async => null;

  @override
  Future<SelectedMedia?> pickFromGallery() async => null;
}
