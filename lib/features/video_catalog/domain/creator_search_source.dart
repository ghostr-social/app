import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

abstract interface class CreatorSearchSource {
  Future<List<ProfileSummary>> searchCreators(String query);
}
