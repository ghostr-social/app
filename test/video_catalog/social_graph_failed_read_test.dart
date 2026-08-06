import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/data/accepted_social_mutations.dart';
import 'package:ghostr/features/social/data/social_graph_task_coordinator.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/nostr_test_values.dart';

void main() {
  test(
    'a failed shared read is removed so the next read can recover',
    () async {
      final coordinator = SocialGraphTaskCoordinator();
      final account = NostrPublicKeyHex.parse(testViewerPublicKey);
      final failure = StateError('relay unavailable');

      await expectLater(
        coordinator.read(
          account,
          SocialGraphMembership.blocked,
          () async => throw failure,
        ),
        throwsA(same(failure)),
      );

      final recovered = await coordinator.read(
        account,
        SocialGraphMembership.blocked,
        () async => {ProfileId.parse('recovered')},
      );

      expect(recovered, {ProfileId.parse('recovered')});
    },
  );
}
