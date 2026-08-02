import 'package:flutter/material.dart';

abstract final class AppSpacing {
  static const double xxs = 4;
  static const double xs = 8;
  static const double sm = 12;
  static const double md = 16;
  static const double lg = 20;
  static const double xl = 24;
  static const double xxl = 28;
  static const double replyIndent = 32;
}

abstract final class AppRadius {
  static const double control = 18;
  static const double media = 24;
}

abstract final class AppSize {
  static const double stateIcon = 42;
  static const double profileAvatar = 38;
}

abstract final class AppPalette {
  static const accentRed = Color(0xFFFF3B5C);
  static const accentBlue = Color(0xFF37D6FF);
  static const night = Color(0xFF090909);
  static const surface = Color(0xFF141414);
  static const altSurface = Color(0xFF1D1D1D);
  static const foreground = Color(0xFFFFFFFF);
  static const mutedForeground = Color(0xB3FFFFFF);
  static const divider = Color(0x1FFFFFFF);
  static const videoBackground = Color(0xFF000000);
  static const videoLoadingBackground = Color(0xFF111111);
  static const videoScrimTop = Color(0x33000000);
  static const videoScrimBottom = Color(0xA6000000);
}
