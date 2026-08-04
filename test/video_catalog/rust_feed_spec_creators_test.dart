import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_spec_builder.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/nostr_test_values.dart';

void main() {
  // ndk parity: NdkVideoRemoteSource skips non-Nostr creator ids and
  // serves const [] when none survive (ndk_video_remote_source.dart).
  test('drops undecodable creator ids and keeps the first valid one', () {
    final spec = buildRustFeedSpec(creatorIds: {
      ProfileId.parse('not-an-npub'),
      ProfileId.parse(testViewerNpub),
    });

    expect(spec?.kind, 'profile');
    expect(spec?.value, testViewerPublicKey);
  });

  test('yields no spec when every creator id is undecodable', () {
    final spec = buildRustFeedSpec(creatorIds: {ProfileId.parse('nope')});

    expect(spec, isNull);
  });

  test('yields no spec for an explicitly empty creator set', () {
    final spec = buildRustFeedSpec(creatorIds: const <ProfileId>{});

    expect(spec, isNull);
  });

  test('the main feed needs a signed-in viewer', () {
    // The Rust main feed is viewer-scoped (api::feed_types FfiFeedSpec).
    expect(() => buildRustFeedSpec(), throwsA(isA<AppFailure>()));
  });
}
