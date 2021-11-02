import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_localized_locales/flutter_localized_locales.dart';

import 'localizations_en.dart';
import 'localizations_zh.dart';

abstract class AppLocalizations {
  AppLocalizations();

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates = [
    delegate,
    GlobalMaterialLocalizations.delegate,
    GlobalCupertinoLocalizations.delegate,
    GlobalWidgetsLocalizations.delegate,
    LocaleNamesLocalizationsDelegate(),
  ];

  static const List<Locale> supportedLocales = [
    Locale('en'),
    Locale('zh'),
  ];

  static Locale lookupLocale(Locale locale) {
    switch (locale.languageCode) {
      case 'zh':
        return Locale('zh');
      default:
        return Locale('en');
    }
  }

  // Common
  String get title;
  String get ok;
  String get saveOk;
  String get cancel;
  String get next;
  String get back;
  String get setting;
  String get search;
  String get info;
  String get friend;
  String get logout;
  String get onlineWaiting;
  String get onlineActive;
  String get onlineSuspend;
  String get onlineLost;
  String get nickname;
  String get bio;
  String get id;
  String get address;
  String get remark;
  String get send;
  String get sended;
  String get resend;
  String get ignore;
  String get agree;
  String get reject;
  String get add;
  String get added;
  String get rejected;
  String get mnemonic;
  String get show;
  String get hide;
  String get change;
  String get download;
  String get wip;
  String get album;
  String get file;
  String get delete;
  String get deleteImmediate;
  String get open;
  String get unknown;
  String get create;
  String get exit;
  String get loadMore;
  String get me;
  String get manager;
  String get block;
  String get blocked;
  String get invite;
  String get emoji;
  String get record;
  String get default0;
  String get others;
  String get closed;
  String get input;
  String get waiting;
  String get notExist;
  String get skip;
  String get register;
  String get gallery;
  String get link;
  String get rename;
  String get moveTo;

  // theme
  String get themeDark;
  String get themeLight;

  // langs
  String get lang;

  // security page (did)
  String get loginChooseAccount;
  String get loginRestore;
  String get loginRestoreOnline;
  String get loginNew;
  String get loginQuick;
  String get newMnemonicTitle;
  String get newMnemonicInput;
  String get hasAccount;
  String get newAccountTitle;
  String get newAccountName;
  String get newAccountPasswrod;
  String get verifyPin;
  String get setPin;
  String get repeatPin;

  // homeage
  String get addFriend;
  String get addService;
  String get sessions;
  String get services;
  String get dataCenter;
  String get devices;
  String get nightly;
  String get scan;

  // friend
  String get contact;
  String get contactIntro;
  String get myQrcode;
  String get qrFriend;
  String get friendInfo;
  String get scanQr;
  String get scanImage;
  String get contactCard;
  String fromContactCard(String name);
  String get setTop;
  String get cancelTop;
  String get unfriend;
  String get waitingRecord;

  // Setting
  String get profile;
  String get preference;
  String get network;
  String get aboutUs;
  String get networkAdd;
  String get networkDht;
  String get networkStable;
  String get networkSeed;
  String get deviceTip;
  String get deviceChangeWs;
  String get deviceChangeHttp;
  String get deviceRemote;
  String get deviceLocal;
  String get deviceQrcode;
  String get deviceQrcodeIntro;
  String get addDevice;
  String get reconnect;
  String get status;
  String get uptime;
  String get days;
  String get hours;
  String get minutes;
  String get memory;
  String get swap;
  String get disk;
  String get about2;
  String get donate;
  String get website;
  String get email;

  // services
  String get files;
  String get filesBio;

  String get assistant;
  String get assistantBio;

  String get groupChat;
  String get groupChats;
  String get groupChatAdd;
  String get groupChatIntro;
  String get groupChatId;
  String get groupChatName;
  String get groupChatKey;
  String get groupChatAddr;
  String get groupChatLocation;
  String get groupChatInfo;
  String get groupChatBio;
  String get groupJoin;
  String get groupCreate;
  String get groupOwner;
  String get groupTypeEncrypted;
  String get groupTypeEncryptedInfo;
  String get groupTypePrivate;
  String get groupTypePrivateInfo;
  String get groupTypeOpen;
  String get groupTypeOpenInfo;
  String get groupCheckTypeAllow;
  String get groupCheckTypeNone;
  String get groupCheckTypeSuspend;
  String get groupCheckTypeDeny;
  String get members;
  String get groupRequireConsent;

  String get domain;
  String get domainIntro;
  String get domainShowProvider;
  String get domainShowName;
  String get domainName;
  String get domainProvider;
  String get domainProviderAdress;
  String get domainAddProvider;
  String get domainSearch;
  String get domainRegisterFailure;
  String get domainSetDefault;
  String get domainSetUnactived;
  String get domainSetActived;
  String get domainDelete;
  String get domainNotDelete;
  String get domainCreateTip;

  String get cloud;
  String get cloudIntro;
  String get star;
  String get document;
  String get image;
  String get music;
  String get video;
  String get trash;
  String get newPost;
  String get newFolder;
  String get uploadFile;
  String get trashClear;
  String get moveTrash;
  String get setstar;
  String get setunstar;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return Future.delayed(
        Duration(seconds: 0), () => _lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) => <String>[
        'en',
        'zh',
      ].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations _lookupAppLocalizations(Locale locale) {
  switch (locale.languageCode) {
    case 'zh':
      {
        return AppLocalizationsZh();
      }
    default:
      {
        return AppLocalizationsEn();
      }
  }
}
