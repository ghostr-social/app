import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_mapper.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';

void main() {
  test('emits profile metadata as a Nostr kind-0 event', () {
    final metadata = ProfileMetadata.parse(
      displayName: 'Nora Relay',
      handle: '@Nora_Relay',
      pictureUrl: 'https://cdn.example/nora.png',
    );

    final event = const NostrProfileMetadataMapper().toEvent(metadata);

    expect(event.kind.value, 0);
    expect(event.tags, isEmpty);
    expect(jsonDecode(event.content), {
      'display_name': 'Nora Relay',
      'name': 'nora_relay',
      'picture': 'https://cdn.example/nora.png',
    });
  });
}
