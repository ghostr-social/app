import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/video_inventory_budget.dart';

void main() {
  test('converts the selected inventory budget into exact bytes', () {
    expect(
      VideoInventoryBudget.twoHundredFiftySixMegabytes.bytes,
      256 * 1024 * 1024,
    );
    expect(VideoInventoryBudget.fourGigabytes.bytes, 4096 * 1024 * 1024);
  });
}
