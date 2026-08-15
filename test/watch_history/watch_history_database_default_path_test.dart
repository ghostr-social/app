import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/watch_history_database.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel('plugins.flutter.io/path_provider');
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  test('opens the native ledger beneath application support', () async {
    final directory = await Directory.systemTemp.createTemp(
      'ghostr-watch-default-',
    );
    debugDefaultTargetPlatformOverride = TargetPlatform.linux;
    messenger.setMockMethodCallHandler(channel, (_) async => directory.path);
    addTearDown(() async {
      debugDefaultTargetPlatformOverride = null;
      messenger.setMockMethodCallHandler(channel, null);
      await directory.delete(recursive: true);
    });

    final database = await openWatchHistoryDatabase();
    await database.close();

    final file = File(
      '${directory.path}${Platform.pathSeparator}ghostr_watch_history.sqlite',
    );
    expect(await file.exists(), isTrue);
  });
}
