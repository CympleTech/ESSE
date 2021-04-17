import 'dart:async';
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/theme.dart';

class Options extends ChangeNotifier {
  Locale locale = Locale('en');
  ThemeMode themeMode = ThemeMode.light;

  void load() async {
    SharedPreferences prefs = await SharedPreferences.getInstance();
    final localeInt  = prefs.getInt('locale');
    final themeInt = prefs.getInt('theme');

    if (localeInt == null) {
      final List<Locale> systemLocales = window.locales;
      if (systemLocales.length > 0) {
        this.locale = AppLocalizations.lookupLocale(systemLocales[0]);
      } else {
        this.locale = Locale('en');
      }
    } else {
      this.locale = LocaleTypeExtension.fromInt(localeInt);
    }

    if (themeInt == null) {
      if (window.platformBrightness == Brightness.dark) {
        this.themeMode = ThemeMode.dark;
      } else {
        this.themeMode = ThemeMode.light;
      }
    } else {
      this.themeMode = ThemeTypeExtension.fromInt(themeInt);
    }

    notifyListeners();
  }

  Future<void> save() async {
    SharedPreferences prefs = await SharedPreferences.getInstance();
    await prefs.setInt('locale', this.locale.toInt());
    await prefs.setInt('theme', this.themeMode.toInt());
  }

  changeLocale(Locale locale) {
    this.locale = locale;
    this.save();
    notifyListeners();
  }

  changeTheme(ThemeMode themeMode) {
    this.themeMode = themeMode;
    this.save();
    notifyListeners();
  }
}

extension ThemeTypeExtension on ThemeMode {
  String localizations(BuildContext context) {
    switch (this) {
      case ThemeMode.dark:
        return AppLocalizations.of(context).themeDark;
      case ThemeMode.light:
        return AppLocalizations.of(context).themeLight;
      default:
        return AppLocalizations.of(context).themeLight;
    }
  }

  int toInt() {
    switch (this) {
      case ThemeMode.light:
        return 0;
      case ThemeMode.dark:
        return 1;
      default:
        return 0;
    }
  }

  static ThemeMode fromInt(int a) {
    switch (a) {
      case 0:
        return ThemeMode.light;
      case 1:
        return ThemeMode.dark;
      default:
        return ThemeMode.light;
    }
  }
}

extension LocaleTypeExtension on Locale {
  String localizations() {
    switch (this) {
      case Locale('en'):
        return 'English';
      case Locale('zh'):
        return '简体中文';
      default:
        return 'English';
    }
  }

  int toInt() {
    switch (this) {
      case Locale('en'):
        return 0;
        case Locale('zh'):
        return 1;
      default:
        return 0;
    }
  }

  static Locale fromInt(int a) {
    switch (a) {
      case 0:
        return Locale('en');
      case 1:
        return Locale('zh');
      default:
        return Locale('en');
    }
  }
}
