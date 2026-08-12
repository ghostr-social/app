import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_app_update.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel('plugins.flutter.io/path_provider');
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  tearDown(() => messenger.setMockMethodCallHandler(channel, null));

  test('stores downloaded updates beneath application support', () async {
    messenger.setMockMethodCallHandler(channel, (call) async {
      expect(call.method, 'getApplicationSupportDirectory');
      return '/private/app-support';
    });
    final environment = ProductionAppUpdateEnvironment.android();

    expect(await environment.directoryPath(), '/private/app-support/updates');

    await environment.dispose();
  });
}
