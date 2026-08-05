import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

const _forbidden = <String>[
  'ndk.requests',
  'ndk.broadcast',
  'ndk.follows',
  'ndk.lists',
  'NdkBroadcastAdapter',
  'NdkNostrEventClient',
  'NdkVideoRemoteSource',
  'WebSocket.connect',
  'WebSocketChannel.connect',
];

const _ndkImportAllowlist = <String>{
  'lib/app/production_nostr_services.dart',
  'lib/features/session/data/ndk_nostr_identity_deriver.dart',
  'lib/features/video_catalog/data/rust_feed_post_mapper.dart',
  'lib/features/video_catalog/data/rust_feed_spec_builder.dart',
  'lib/platform/nostr/blossom_upload_result_mapper.dart',
  'lib/platform/nostr/build_ndk.dart',
  'lib/platform/nostr/ndk_blossom_video_uploader.dart',
  'lib/platform/nostr/ndk_nostr_session.dart',
  'lib/platform/nostr/ndk_nostr_social.dart',
  'lib/platform/nostr/rust_nostr_event_client.dart',
  'lib/platform/nostr/rust_nostr_event_mapper.dart',
  'lib/platform/nostr/signed_nostr_event_json.dart',
};

void main() {
  test('production Dart never owns Nostr relay communication', () {
    final violations =
        _dartSources().expand(_violations).toList(growable: false)..sort();

    expect(
      violations,
      isEmpty,
      reason: 'Rust is the only production Nostr transport.',
    );
  });

  test('NDK stays inside local signing parsing and Blossom adapters', () {
    final imports = _dartSources()
        .where((file) => file.readAsStringSync().contains('package:ndk/'))
        .map((file) => file.path)
        .toSet();

    expect(imports, _ndkImportAllowlist);
  });

  test('production composition exposes no NDK transport escape hatch', () {
    final source =
        File('lib/app/production_nostr_services.dart').readAsStringSync();

    expect(source, isNot(contains('final Ndk ndk;')));
    expect(source, isNot(contains('SignedEventBroadcastPort? broadcast')));
  });
}

Iterable<File> _dartSources() {
  return Directory('lib')
      .listSync(recursive: true)
      .whereType<File>()
      .where((file) => file.path.endsWith('.dart'))
      .where((file) => !file.path.contains('/src/rust/'));
}

Iterable<String> _violations(File file) sync* {
  final source = file.readAsStringSync();
  for (final symbol in _forbidden) {
    if (source.contains(symbol)) yield '${file.path}: $symbol';
  }
}
