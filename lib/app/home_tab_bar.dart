import 'package:flutter/material.dart';
import 'package:ghostr/app/home_tab.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class HomeTabBar extends StatelessWidget {
  const HomeTabBar({
    required this.currentTab,
    required this.onSelect,
    super.key,
  });

  final HomeTab currentTab;
  final ValueChanged<int> onSelect;

  @override
  Widget build(BuildContext context) {
    return BottomNavigationBar(
      currentIndex: HomeTab.values.indexOf(currentTab),
      type: BottomNavigationBarType.fixed,
      backgroundColor: Theme.of(context).scaffoldBackgroundColor,
      selectedItemColor: Theme.of(context).colorScheme.primary,
      unselectedItemColor: AppPalette.mutedForeground,
      onTap: onSelect,
      items: HomeTab.values
          .map(
            (tab) => BottomNavigationBarItem(
              icon: Icon(tab.icon),
              label: tab.label,
            ),
          )
          .toList(),
    );
  }
}
