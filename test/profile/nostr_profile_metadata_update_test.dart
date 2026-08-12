import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_mapper.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';

void main() {
  test('profile update replaces owned fields and preserves unrelated JSON', () {
    const previous =
        '{"display_name":"Old","name":"old",'
        '"picture":"https://old.example/avatar",'
        '"about":"Still here","nip05":"nora@example.com"}';
    final metadata = ProfileMetadata.parse(
      displayName: 'Nora Relay',
      handle: 'nora',
      pictureUrl: 'https://new.example/avatar',
    );

    final event = const NostrProfileMetadataMapper().toEvent(
      metadata,
      previousContent: previous,
    );

    expect(jsonDecode(event.content), {
      'display_name': 'Nora Relay',
      'name': 'nora',
      'picture': 'https://new.example/avatar',
      'about': 'Still here',
      'nip05': 'nora@example.com',
    });
  });
}
