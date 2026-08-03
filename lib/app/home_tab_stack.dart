import 'package:flutter/material.dart';
import 'package:ghostr/app/home_tab.dart';

/// Keeps every visited tab mounted so switching back is instant, while
/// unvisited tabs stay unbuilt.
class HomeTabStack extends StatelessWidget {
  const HomeTabStack({
    required this.currentTab,
    required this.visitedTabs,
    required this.tabBuilder,
    super.key,
  });

  final HomeTab currentTab;
  final Set<HomeTab> visitedTabs;
  final Widget Function(HomeTab tab) tabBuilder;

  @override
  Widget build(BuildContext context) {
    return IndexedStack(
      index: HomeTab.values.indexOf(currentTab),
      children: HomeTab.values.map(_tabScreen).toList(),
    );
  }

  Widget _tabScreen(HomeTab tab) {
    if (!visitedTabs.contains(tab)) return const SizedBox.shrink();
    return KeyedSubtree(
      key: ValueKey('home-tab-${tab.name}'),
      child: tabBuilder(tab),
    );
  }
}
