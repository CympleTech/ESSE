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
  String get contact => 'Contacts';
  @override
  String get friend => 'Friend';
  @override
  String get friends => 'Friends';
  @override
  String get logout => 'Logout';
  @override
  String get onlineWaiting => 'Waiting...';
  @override
  String get onlineActive => 'Active';
  @override
  String get onlineSuspend => 'Suspend';
  @override
  String get onlineLost => 'Offline';
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
  @override
  String get create => 'Create';
  @override
  String get exit => 'Exit';
  @override
  String get loadMore => 'Load more...';
  @override
  String get me => 'Me';
  @override
  String get manager => 'Manager';
  @override
  String get block => 'Block';
  @override
  String get blocked => 'Blocked';
  @override
  String get invite => 'Invite';
  @override
  String get emoji => 'Emoji';
  @override
  String get record => 'Record';
  @override
  String get others => 'Others';
  @override
  String get closed => 'Closed';

  // theme
  @override
  String get themeDark => 'Dark';
  @override
  String get themeLight => 'Light';

  // langs
  @override
  String get lang => 'Language';

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
  String get addService => 'Add Service';
  @override
  String get services => 'Services';
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
  String get qrFriend => 'Scan for Add Friend';
  @override
  String get friendInfo => 'Friend Info';
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
  String get deviceRemote => 'Remote';
  @override
  String get deviceLocal => 'Local';
  @override
  String get deviceQrcode => 'Device QR Code';
  @override
  String get addDevice => 'Add Device';
  @override
  String get reconnect => 'Re-Connect';
  @override
  String get status => 'View Status';
  @override
  String get uptime => 'Uptime';
  @override
  String get days => 'Days';
  @override
  String get hours => 'Hours';
  @override
  String get minutes => 'Minutes';
  @override
  String get memory => 'Memory';
  @override
  String get swap => 'Swap';
  @override
  String get disk => 'Disk';
  @override
  String get deviceQrcodeIntro => 'Tips: Scan to Login and sync, use it with care, and do not tell others';
  @override
  String get about1 => 'ESSE (Encrypted Symmetrical Session Engine)';
  @override
  String get about2 => 'An open source encrypted peer-to-peer session system would allow data to be sent securely from one terminal to another without going through third-party services.';
  @override
  String get donate => 'Donate';
  @override
  String get website => 'Website';
  @override
  String get email => 'Email';

  // services
  @override
  String get files => 'Files';
  @override
  String get filesBio => 'Sync & manager files between devices';
  @override
  String get assistant => 'Jarvis';
  @override
  String get assistantBio => 'Jarvis is a robot, only belongs to you.';
  @override
  String get groupChat => 'Group Chat';
  @override
  String get groupChats => 'Groups';
  @override
  String get groupChatAdd => 'Add Group Chat';
  @override
  String get groupChatIntro => 'Multiple group chats';
  @override
  String get groupChatId => 'Group ID';
  @override
  String get groupChatName => 'Group Name';
  @override
  String get groupChatKey => 'Encrypted Key';
  @override
  String get groupChatAddr => 'Group Address';
  @override
  String get groupChatInfo => 'Group Information';
  @override
  String get groupChatBio => 'Group Bio';
  @override
  String get groupJoin => 'Join Open Group';
  @override
  String get groupCreate => 'Create Group';
  @override
  String get groupOwner => 'Owner';
  @override
  String get groupTypeEncrypted => 'Encrypted';
  @override
  String get groupTypeEncryptedInfo => "Encrypted: It can only be joined by the invitation of members, and the manager's consent is optional, Members hold a zero-knowledge proof about the key to enter; Group information and messages are encrypted and stored on the server, and the server NOT has secret key, INVISIBLE information.";
  @override
  String get groupTypePrivate => 'Private';
  @override
  String get groupTypePrivateInfo => "Private: It can only be joined by the invitation of members, and the manager's consent is optional; Group information and messages are stored on the server, VISIBLE information.";
  @override
  String get groupTypeOpen => 'Open';
  @override
  String get groupTypeOpenInfo => 'Open: Any member who has group information can freely enter and leave; Group information and messages are stored on the server, VISIBLE information.';
  @override
  String get groupCheckTypeAllow => 'You can create a new group chat';
  @override
  String get groupCheckTypeNone => 'Restricted, the allowed number is full';
  @override
  String get groupCheckTypeSuspend => 'Account is suspended';
  @override
  String get groupCheckTypeDeny => 'No permission to create here';
  @override
  String get members => 'Members';
  @override
  String get groupRequireConsent => "Requires manager's consent";
}
