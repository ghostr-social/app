import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

const _feedWorkflows = <String>[
  'feed_backfill.dart',
  'feed_engagement.dart',
  'feed_fetcher.dart',
  'feed_interaction_reconciler.dart',
  'feed_loads.dart',
  'feed_pagination.dart',
  'feed_session.dart',
];

void main() {
  test('feed business workflows stay in the framework-free inner layer', () {
    for (final module in _feedWorkflows) {
      expect(
        File('lib/features/video_catalog/presentation/$module').existsSync(),
        isFalse,
        reason: '$module owns feed decisions rather than rendered state.',
      );
      final source = File(
        'lib/features/video_catalog/domain/use_cases/$module',
      ).readAsStringSync();
      expect(source, isNot(contains('package:flutter/')));
      expect(source, isNot(contains('/presentation/')));
    }
  });

  test('progressive playback widget delegates gateway workflow ownership', () {
    final surface = File(
      'lib/platform/media/gateway_video_playback_surface.dart',
    ).readAsStringSync();

    expect(surface, isNot(contains('._gateway')));
    expect(surface, isNot(contains('_loadGatewayMedia')));
    expect(
      File(
        'lib/platform/media/gateway_playback_cubit.dart',
      ).existsSync(),
      isTrue,
    );
  });
}
