import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_evidence_models.dart';

void main() {
  test('WARP evidence decoder rejects an unknown schema', () {
    expect(
      () => WarpEvidencePage.parse('{"schema_version":2}'),
      throwsFormatException,
    );
  });
}
