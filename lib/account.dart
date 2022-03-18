import 'package:flutter/material.dart';

import 'dart:ui' show Locale;
import 'dart:convert';
import 'dart:typed_data';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/avatar.dart';

final MNEMONIC_LANGUAGE = [
  Language.English,
  Language.SimplifiedChinese,
  Language.TraditionalChinese,
  Language.French,
  Language.Italian,
  Language.Japanese,
  Language.Korean,
  Language.Spanish,
  Language.Portuguese,
  Language.Czech,
];

enum Language {
  English,
  SimplifiedChinese,
  TraditionalChinese,
  Czech,
  French,
  Italian,
  Japanese,
  Korean,
  Spanish,
  Portuguese,
}

extension LanguageExtension on Language {
  String localizations(BuildContext context) {
    switch (this) {
      case Language.English:
        return AppLocalizations.of(context).english;
      case Language.SimplifiedChinese:
        return AppLocalizations.of(context).simplifiedChinese;
      case Language.TraditionalChinese:
        return AppLocalizations.of(context).traditionalChinese;
      case Language.Czech:
        return AppLocalizations.of(context).czech;
      case Language.French:
        return AppLocalizations.of(context).french;
      case Language.Italian:
        return AppLocalizations.of(context).italian;
      case Language.Japanese:
        return AppLocalizations.of(context).japanese;
      case Language.Korean:
        return AppLocalizations.of(context).korean;
      case Language.Spanish:
        return AppLocalizations.of(context).spanish;
      case Language.Portuguese:
        return AppLocalizations.of(context).portuguese;
    }
  }

  int toInt() {
    switch (this) {
      case Language.English:
        return 0;
      case Language.SimplifiedChinese:
        return 1;
      case Language.TraditionalChinese:
        return 2;
      case Language.Czech:
        return 3;
      case Language.French:
        return 4;
      case Language.Italian:
        return 5;
      case Language.Japanese:
        return 6;
      case Language.Korean:
        return 7;
      case Language.Spanish:
        return 8;
      case Language.Portuguese:
        return 9;
    }
  }

  static Language fromInt(int a) {
    switch (a) {
      case 0:
        return Language.English;
      case 1:
        return Language.SimplifiedChinese;
      case 2:
        return Language.TraditionalChinese;
      case 3:
        return Language.Czech;
      case 4:
        return Language.French;
      case 5:
        return Language.Italian;
      case 6:
        return Language.Japanese;
      case 7:
        return Language.Korean;
      case 8:
        return Language.Spanish;
      case 9:
        return Language.Portuguese;
      default:
        return Language.English;
    }
  }

  static Language fromLocale(Locale locale) {
    switch (locale.languageCode) {
      case 'en':
        return Language.English;
      case 'zh':
        return Language.SimplifiedChinese;
      default:
        return Language.English;
    }
  }
}

class Account {
  String pid = '';
  String name = '';
  Uint8List? avatar;
  String pin = '';

  Account(String pid, String name, [String avatar = "", String pin = ""]) {
    this.pid = pid;
    this.name = name;
    this.updateAvatar(avatar);
    this.pin = pin;
  }

  String encodeAvatar() {
    if (this.avatar != null && this.avatar!.length > 1) {
      return base64.encode(this.avatar!);
    } else {
      return '';
    }
  }

  void updateAvatar(String avatar) {
    if (avatar.length > 1) {
      this.avatar = base64.decode(avatar);
    } else {
      this.avatar = null;
    }
  }

  Avatar showAvatar({double width = 45.0}) {
    return Avatar(
      width: width,
      name: this.name,
      avatar: this.avatar,
    );
  }
}
