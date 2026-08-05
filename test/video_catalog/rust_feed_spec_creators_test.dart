import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_spec_builder.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('drops undecodable creator ids and keeps the decodable ones', () {
    final spec = buildRustFeedSpec(creatorIds: {
      ProfileId.parse('not-an-npub'),
      ProfileId.parse(testViewerNpub),
    });

    expect(spec?.kind, FfiFeedKind.profile);
    expect(spec?.creators, [testViewerPublicKey]);
  });

  // The Following feed asks for every follow at once
  // (filtered_video_feed_repository.dart passes the whole set), so a
  // spec naming one creator would query and filter to a single author.
  test('names every decoded creator, not just the first', () {
    final spec = buildRustFeedSpec(creatorIds: {
      ProfileId.parse(testViewerNpub),
      ProfileId.parse(testCreatorNpub),
    });

    expect(
      spec?.creators,
      unorderedEquals([testViewerPublicKey, testCreatorPublicKey]),
    );
  });

  test('yields no spec when every creator id is undecodable', () {
    final spec = buildRustFeedSpec(creatorIds: {ProfileId.parse('nope')});

    expect(spec, isNull);
  });

  test('yields no spec for an explicitly empty creator set', () {
    final spec = buildRustFeedSpec(creatorIds: const <ProfileId>{});

    expect(spec, isNull);
  });
}
