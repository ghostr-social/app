import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/app_update/android_app_update_adapter.dart';
import 'package:ghostr/platform/app_update/android_app_update_platform.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('translates Android channel failures at the domain boundary', () async {
    const channel = MethodChannel(AndroidAppUpdatePlatform.channelName);
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    messenger.setMockMethodCallHandler(channel, (_) async {
      throw PlatformException(code: 'offline');
    });
    addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
    final platform = AndroidAppUpdatePlatform(channel: channel);
    final adapter = AndroidAppUpdateAdapter(platform);
    addTearDown(adapter.dispose);

    expect(
      adapter.readPermission(),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Android app updates are unavailable.',
        ),
      ),
    );
  });
}
