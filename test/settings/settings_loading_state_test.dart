import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';

import '../support/settings_screen_harness.dart';

void main() {
  testWidgets('announces the settings loading state', (tester) async {
    await tester.pumpWidget(
      settingsScreenHarness(_PendingSettingsRepository()),
    );
    await tester.pump();

    expect(find.bySemanticsLabel('Loading settings'), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
  });
}

class _PendingSettingsRepository implements AppSettingsRepository {
  final _load = Completer<AppSettings>();

  @override
  Future<AppSettings> load() => _load.future;

  @override
  Future<void> save(AppSettings settings) async {}
}
