import 'localizations.dart';

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn();

  @override
  String get title => 'Encrypted Secure Session Engine.';
  @override
  String get ok => 'OK';
  @override
  String get cancel => 'Cancel';
  @override
  String get next => 'Next';
  @override
  String get back => 'Back';
  @override
  String get setting => 'Setting';
  @override
  String get search => 'Search';
  @override
  String get info => 'Info';
  @override
  String get contact => 'Contact';
  @override
  String get friend => 'Friend';
  @override
  String get logout => 'Logout';
  @override
  String get online => 'Online';
  @override
  String get offline => 'Offline';
  @override
  String get nickname => 'Name';
  @override
  String get id => 'ID';
  @override
  String get address => 'Address';
  @override
  String get remark => 'Remark';
  @override
  String get send => 'Send';
  @override
  String get sended => 'Sended';
  @override
  String get resend => 'Resend';
  @override
  String get ignore => 'Ignore';
  @override
  String get agree => 'Agree';
  @override
  String get add => 'Add';
  @override
  String get added => 'Added';
  @override
  String get reject => 'Reject';
  @override
  String get rejected => 'Rejected';
  @override
  String get mnemonic => 'Mnemonic';
  @override
  String get show => 'Show';
  @override
  String get hide => 'Hide';
  @override
  String get change => 'Change';
  @override
  String get download => 'Download';
  @override
  String get wip => 'Work-in-Progress';
  @override
  String get album => 'Album';
  @override
  String get file => 'Files';
  @override
  String get delete => 'Delete';
  @override
  String get open => 'Open';
  @override
  String get unknown => 'Unknown';

  // theme
  @override
  String get themeDark => 'Dark';
  @override
  String get themeLight => 'Light';

  // langs
  @override
  String get lang => 'Language';
  @override
  String get langEn => 'English';
  @override
  String get langZh => 'Chinese';

  // security page (did)
  @override
  String get loginChooseAccount => 'Choose account';
  @override
  String get loginRestore => 'Restore account';
  @override
  String get loginRestoreOnline => 'Restore from online';
  @override
  String get loginNew => 'Create Account';
  @override
  String get newMnemonicTitle => 'Mnemonic code (DID)';
  @override
  String get newMnemonicInput => 'Generate';
  @override
  String get hasAccount => 'Has account ? Login';
  @override
  String get newAccountTitle => 'Account Info';
  @override
  String get newAccountName => 'Type Name';
  @override
  String get newAccountPasswrod => 'Type lock password';
  @override
  String get verifyPin => 'Verify PIN';
  @override
  String get setPin => 'Set PIN';
  @override
  String get repeatPin => 'Repeat PIN';

  // home page
  @override
  String get addFriend => 'Add Friend';
  @override
  String get addGroup => 'Add Service';
  @override
  String get chats => 'Sessions';
  @override
  String get groups => 'Services';
  @override
  String get files => 'Files';
  @override
  String get devices => 'Devices';
  @override
  String get nightly => 'Night Mode';
  @override
  String get scan => 'Scan';

  // friend
  @override
  String get myQrcode => 'My QRCode';
  @override
  String get qrFriend => 'Scan for friend';
  @override
  String get scanQr => 'Scan Qrcode';
  @override
  String get scanImage => 'Scan Image';
  @override
  String get contactCard => 'Contract Card';
  @override
  String fromContactCard(String name) => "Contract card from ${name} shared.";
  @override
  String get setTop => 'Add to home';
  @override
  String get cancelTop => 'Cancal';
  @override
  String get unfriended => 'Unfriended';
  @override
  String get unfriend => 'Unfriend';
  @override
  String get waitingRecord => 'Waiting to record';

  // Setting
  @override
  String get profile => 'Profile';
  @override
  String get preference => 'Preference';
  @override
  String get network => 'Network';
  @override
  String get aboutUs => 'About Us';
  @override
  String get networkAdd => 'Add bootstrap';
  @override
  String get networkDht => 'Network Connections';
  @override
  String get networkStable => 'Stable Connections';
  @override
  String get networkSeed => 'Bootstrap seeds';
  @override
  String get deviceTip => "Tips: Please make sure you know the meaning of these settings before changing it";
  @override
  String get deviceChangeWs => 'Change websocket';
  @override
  String get deviceChangeHttp => 'Change http';
  @override
  String get deviceLocal => 'Local';
  @override
  String get deviceQrcode => 'Device QR Code';
  @override
  String get deviceQrcodeIntro => 'Tips: Scan to Login and sync, use it with care, and do not tell others';
  @override
  String get openSource => 'Open Source Power By CympleTech.';
  @override
  String get tdnBased => 'A Distributed Network and Application build on TDN.';
  @override
  String get donate => 'Donate';
  @override
  String get website => 'Website';
  @override
  String get email => 'Email';
}
