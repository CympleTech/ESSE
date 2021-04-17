import 'localizations.dart';

/// The translations for English (`en`).
class AppLocalizationsZh extends AppLocalizations {
  AppLocalizationsZh();

  @override
  String get title => '加密安全会话引擎。';
  @override
  String get ok => '确认';
  @override
  String get cancel => '取消';
  @override
  String get next => '下一步';
  @override
  String get back => '返回';
  @override
  String get setting => '设置';
  @override
  String get search => '搜索';
  @override
  String get info => '信息';
  @override
  String get contact => '联系人';
  @override
  String get friend => '好友';
  @override
  String get logout => '退出';
  @override
  String get online => '在线';
  @override
  String get offline => '离线';
  @override
  String get nickname => '昵称';
  @override
  String get id => '身份账户';
  @override
  String get address => '网络地址';
  @override
  String get remark => '备注';
  @override
  String get send => '发送';
  @override
  String get sended => '已发送';
  @override
  String get resend => '重新发送';
  @override
  String get ignore => '忽略';
  @override
  String get agree => '同意';
  @override
  String get add => '添加';
  @override
  String get added => '已添加';
  @override
  String get reject => '拒绝';
  @override
  String get rejected => '已拒绝';
  @override
  String get mnemonic => '助记词';
  @override
  String get show => '显示';
  @override
  String get hide => '隐藏';
  @override
  String get change => '修改';
  @override
  String get download => '下载';
  @override
  String get wip => '即将开放';
  @override
  String get album => '图片';
  @override
  String get file => '文件';
  @override
  String get delete => '删除';
  @override
  String get open => '打开';
  @override
  String get unknown => '未知';

  // theme
  @override
  String get themeDark => '深色';
  @override
  String get themeLight => '浅色';

  // langs
  @override
  String get lang => '语言';

  // security page (did)
  @override
  String get loginChooseAccount => '选择账户';
  @override
  String get loginRestore => '恢复账户';
  @override
  String get loginRestoreOnline => '账户在线恢复';
  @override
  String get loginNew => '新建账户';
  @override
  String get newMnemonicTitle => '助记词（DID）';
  @override
  String get newMnemonicInput => '生成';
  @override
  String get hasAccount => '已有账户，直接登录';
  @override
  String get newAccountTitle => '账号信息';
  @override
  String get newAccountName => '请输入昵称';
  @override
  String get newAccountPasswrod => '请设置解锁密码';
  @override
  String get verifyPin => '验证 PIN';
  @override
  String get setPin => '设置 PIN';
  @override
  String get repeatPin => '确认 PIN';

  // home page
  @override
  String get addFriend => '添加好友';
  @override
  String get addGroup => '添加服务';
  @override
  String get chats => '聊天列表';
  @override
  String get groups => '服务列表';
  @override
  String get files => '文件管理';
  @override
  String get devices => '关联设备';
  @override
  String get nightly => '夜间模式';
  @override
  String get scan => '扫一扫';

  // friend
  @override
  String get myQrcode => '我的二维码';
  @override
  String get qrFriend => '扫二维码加好友';
  @override
  String get scanQr => '扫描二维码';
  @override
  String get scanImage => '识别图片';
  @override
  String get contactCard => '名片';
  @override
  String fromContactCard(String name) => "来自 ${name} 的名片分享。";
  @override
  String get setTop => '置于首页';
  @override
  String get cancelTop => '取消首页';
  @override
  String get unfriended => '已解除好友';
  @override
  String get unfriend => '解除好友';
  @override
  String get waitingRecord => '等待录音';

  // Setting
  @override
  String get profile => '个人信息';
  @override
  String get preference => '偏好设置';
  @override
  String get network => '网络状况';
  @override
  String get aboutUs => '关于我们';
  @override
  String get networkAdd => '添加网络种子';
  @override
  String get networkDht => '网络连接';
  @override
  String get networkStable => '稳定连接';
  @override
  String get networkSeed => '网络种子';
  @override
  String get deviceTip => '友情提醒：更改前请确保您了解这些设置的含义';
  @override
  String get deviceChangeWs => '修改 Websocket 接口';
  @override
  String get deviceChangeHttp => '修改 Http 接口';
  @override
  String get deviceLocal => '本机';
  @override
  String get deviceQrcode => '设备二维码';
  @override
  String get addDevice => '添加设备';
  @override
  String get reconnect => '重连';
  @override
  String get status => '查看状态';
  @override
  String get uptime => '已运行';
  @override
  String get days => '天';
  @override
  String get hours => '小时';
  @override
  String get minutes => '分钟';
  @override
  String get memory => '内存';
  @override
  String get swap => '虚拟内存';
  @override
  String get disk => '硬盘';
  @override
  String get deviceQrcodeIntro => '扫码登陆与同步账户，小心使用，请勿告知他人';
  @override
  String get openSource => '开源代码由 CympleTech 公司组织开发。';
  @override
  String get tdnBased => '一款基于 TDN 的分布式网络和应用程序。';
  @override
  String get donate => '捐助';
  @override
  String get website => '官网';
  @override
  String get email => '联系邮件';
}
