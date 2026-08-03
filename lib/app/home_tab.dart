import 'package:flutter/material.dart';

enum HomeTab {
  home('Home', Icons.home_rounded),
  search('Search', Icons.search_rounded),
  create('Create', Icons.add_box_rounded),
  activity('Activity', Icons.notifications_rounded),
  profile('Profile', Icons.person_rounded);

  const HomeTab(this.label, this.icon);

  final String label;
  final IconData icon;
}
