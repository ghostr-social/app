import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_offline_restart_manifest.dart';
import '../../integration_test/support/warp_offline_restart_snapshot.dart';

void main() {
  test('recognizes the exact viewer and event durable commit', () async {
    final directory = await Directory.systemTemp.createTemp('warp-snapshot-');
    addTearDown(() => directory.delete(recursive: true));
    final file = File('${directory.path}/nostr-event-cache-v1.json');
    final manifest = WarpOfflineRestartManifest(
      eventId: 'event-1',
      originPort: 8080,
      relay: Uri.parse('ws://127.0.0.1:9000'),
      viewerPublicKey: 'viewer-1',
    );
    await file.writeAsString(
      jsonEncode({
        'version': 1,
        'viewer': {'scope': 'signed_in', 'public_key': 'viewer-1'},
        'events': [
          {'id': 'event-1'},
        ],
      }),
      flush: true,
    );

    expect(warpOfflineSnapshotCommitted(file, manifest), isTrue);
  });
}
