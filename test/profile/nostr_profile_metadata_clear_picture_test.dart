import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_mapper.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';

void main() {
  test('clearing a picture removes it from existing Nostr metadata', () {
    final metadata = ProfileMetadata.parse(
      displayName: 'Nora Relay',
      handle: 'nora',
    );

    final event = const NostrProfileMetadataMapper().toEvent(
      metadata,
      previousContent: '{"picture":"https://old.example/avatar.png"}',
    );

    expect(jsonDecode(event.content), {
      'display_name': 'Nora Relay',
      'name': 'nora',
    });
  });
}
