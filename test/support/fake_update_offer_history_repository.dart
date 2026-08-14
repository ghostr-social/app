import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/update_offer_history_repository.dart';

final class FakeUpdateOfferHistoryRepository
    implements UpdateOfferHistoryRepository {
  FakeUpdateOfferHistoryRepository({this.readFailure, this.writeFailure});

  AndroidVersionCode? lastDeclined;
  Future<void>? beforeRead;
  Future<void>? beforeWrite;
  final Object? readFailure;
  final Object? writeFailure;
  int reads = 0;
  int writes = 0;

  @override
  Future<AndroidVersionCode?> readLastDeclinedVersion() async {
    reads += 1;
    await beforeRead;
    if (readFailure != null) throw readFailure!;
    return lastDeclined;
  }

  @override
  Future<void> recordDeclinedVersion(AndroidVersionCode version) async {
    writes += 1;
    await beforeWrite;
    if (writeFailure != null) throw writeFailure!;
    final current = lastDeclined;
    if (current == null || version.compareTo(current) > 0) {
      lastDeclined = version;
    }
  }
}
