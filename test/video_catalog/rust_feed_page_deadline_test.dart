import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_page_reader.dart';

void main() {
  // rust/src/discovery/search_queries.rs: DISCOVERY_QUERY_TIMEOUT is the
  // slowest query a plan can contain, and a page becomes visible only
  // once the whole plan resolves. A deadline at or below that answers
  // before the only revision that will ever carry the page.
  test('outlives the slowest query the Rust pipeline plans', () {
    expect(
      rustFeedPageDeadline,
      greaterThan(rustDiscoveryQueryTimeout),
      reason: 'the deadline is a safety net, not the normal exit path',
    );
    expect(rustDiscoveryQueryTimeout, const Duration(seconds: 8));
  });
}
