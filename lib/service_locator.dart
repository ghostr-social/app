import 'package:get_it/get_it.dart';
import 'package:ghostr/screens/feed_viewmodel.dart';

import 'configs/base.dart';

final locator = GetIt.instance;

void setup(String address) {
  locator.registerSingleton<FeedViewModel>(FeedViewModel());
  baseLoopbackUrl =  "http://$address";

}
