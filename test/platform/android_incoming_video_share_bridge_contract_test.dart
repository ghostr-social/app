import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Android safely bridges cold and warm shared videos', () {
    final sourceRoot = 'android/app/src/main/kotlin/app/ghostr';
    final activity = File('$sourceRoot/MainActivity.kt').readAsStringSync();
    final bridge = File(
      '$sourceRoot/IncomingVideoShareBridge.kt',
    ).readAsStringSync();
    final receiver = File(
      '$sourceRoot/IncomingVideoShareReceiver.kt',
    ).readAsStringSync();
    final cacheFile = File('$sourceRoot/IncomingVideoShareCache.kt');
    final cache = cacheFile.existsSync() ? cacheFile.readAsStringSync() : '';
    final store = File(
      '$sourceRoot/IncomingVideoShareStore.kt',
    ).readAsStringSync();
    final worker = File(
      '$sourceRoot/IncomingVideoShareWorker.kt',
    ).readAsStringSync();

    final setIntent = activity.indexOf('setIntent(intent)');
    final receive = activity.indexOf('receive(intent,');
    expect(setIntent, greaterThanOrEqualTo(0));
    expect(receive, greaterThan(setIntent));
    expect(activity, isNot(contains('shareIntentCaptured')));
    expect(activity, contains('IncomingVideoShareActivityLifecycle('));
    expect(activity, contains('shareLifecycle.configureEngine()'));
    expect(activity, contains('shareLifecycle.receive(intent)'));
    expect(activity, contains('shareLifecycle.savedCaptureId'));
    expect(activity, contains('shareLifecycle.acknowledge(generation'));
    expect(activity, contains('acknowledgeShare'));
    expect(activity, contains('Intent(Intent.ACTION_MAIN)'));
    expect(bridge, contains('app.ghostr/incoming_video_share'));
    expect(bridge, contains('takePendingVideo'));
    expect(bridge, contains('releaseVideo'));
    expect(bridge, contains('acknowledgeVideo'));
    expect(bridge, contains('restorePending'));
    expect(bridge, contains('videoAvailable'));
    expect(bridge, contains('worker.execute { bootstrap'));
    expect(bridge, contains('IncomingVideoShareWorker'));
    expect(worker, contains('Executors.newSingleThreadExecutor'));
    expect(bridge, contains('MAX_PENDING_DELIVERIES'));
    expect(bridge, contains('clearPendingDeliveries'));
    expect(bridge, contains('releaseDelivery'));
    expect(bridge, isNot(contains('delivery.share?.delete()')));
    expect(receiver, contains('ContentResolver.SCHEME_CONTENT'));
    expect(receiver, contains('contentResolver.getType(uri)'));
    expect(receiver, contains('OpenableColumns.SIZE'));
    expect(receiver, contains('deleteStaleCacheFiles'));
    expect(cache, contains('MAX_COPY_BYTES'));
    expect(cache, contains('STALE_FILE_AGE_MILLIS'));
    expect(cache, contains('canonicalFile'));
    expect(cache, contains('startsWith(CACHE_PREFIX)'));
    expect(store, contains('SharedPreferences'));
    expect(store, contains('sourceKey'));
  });
}
