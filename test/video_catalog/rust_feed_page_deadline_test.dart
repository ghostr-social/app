import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_page_reader.dart';

void main() {
  test('outlives discovery plus sequential profile enrichment', () {
    final planned =
        rustDiscoveryQueryTimeout + rustProfileEnrichmentQueryTimeout;
    expect(
      rustFeedPageDeadline,
      greaterThan(planned),
      reason: 'the deadline is a safety net, not the normal exit path',
    );
    expect(rustDiscoveryQueryTimeout, const Duration(seconds: 8));
    expect(rustProfileEnrichmentQueryTimeout, const Duration(seconds: 5));
  });
}
