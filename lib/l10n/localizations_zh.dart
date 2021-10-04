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
  String get friends => '好友';
  @override
  String get logout => '退出';
  @override
  String get onlineWaiting => '连接中...';
  @override
  String get onlineActive => '在线';
  @override
  String get onlineSuspend => '挂起';
  @override
  String get onlineLost => '离线';
  @override
  String get nickname => '昵称';
  @override
  String get bio => '个性签名';
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
  @override
  String get create => '创建';
  @override
  String get exit => '退出';
  @override
  String get loadMore => '加载更多';
  @override
  String get me => '我';
  @override
  String get manager => '管理员';
  @override
  String get block => '拉黑';
  @override
  String get blocked => '已拉黑';
  @override
  String get invite => '邀请';
  @override
  String get emoji => '动画表情';
  @override
  String get record => '语音';
  @override
  String get default0 => '默认';
  @override
  String get others => '其他';
  @override
  String get closed => '已关闭';
  @override
  String get input => '输入信息';
  @override
  String get waiting => '等待中';
  @override
  String get notExist => '用户不存在。';

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
  String get addService => '添加服务';
  @override
  String get services => '服务';
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
  String get qrFriend => '扫二维码添加好友';
  @override
  String get friendInfo => '好友信息';
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
  String get deviceRemote => '远程';
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
  String get about1 => 'ESSE (加密对称会话引擎)';
  @override
  String get about2 => '一款开源的加密对等通信系统，允许信息安全地从发送端经由网络直接到达接收端而不用经过第三方服务。';
  @override
  String get donate => '捐助';
  @override
  String get website => '官网';
  @override
  String get email => '联系邮件';

  // services
  @override
  String get files => '文件管理';
  @override
  String get filesBio => '同步和管理各设备上的文件';
  @override
  String get assistant => 'Jarvis';
  @override
  String get assistantBio => 'Jarvis 是个机器人，只属于你。';
  @override
  String get groupChat => '群聊';
  @override
  String get groupChats => '群聊';
  @override
  String get groupChatAdd => '添加群聊';
  @override
  String get groupChatIntro => '各种各样的群聊';
  @override
  String get groupChatId => '群ID';
  @override
  String get groupChatAddr => '群聊地址';
  @override
  String get groupChatName => '群名称';
  @override
  String get groupChatKey => '加密密码';
  @override
  String get groupChatInfo => '群聊信息';
  @override
  String get groupChatBio => '群公告';
  @override
  String get groupJoin => '加入公开群';
  @override
  String get groupCreate => '新建群';
  @override
  String get groupOwner => '群主';
  @override
  String get groupTypeEncrypted => '加密';
  @override
  String get groupTypeEncryptedInfo => '加密的群聊：只能通过群成员邀请加入，可选是否需要管理员同意，成员需持有密钥的零知识证明方可进入；群信息和消息全部加密存储在服务端，服务端无密钥，不可见信息。';
  @override
  String get groupTypePrivate => '私有';
  @override
  String get groupTypePrivateInfo => '私有的群聊：只能通过群成员邀请加入，可选是否需要管理员同意；群信息和消息存储在服务端，服务端可见信息。';
  @override
  String get groupTypeOpen => '公开';
  @override
  String get groupTypeOpenInfo => '公开的群聊：任何拥有群信息的成员均可自由进出；群信息和消息存储在服务端，服务端可见信息。';
  @override
  String get groupCheckTypeAllow => '可以创建新的群聊';
  @override
  String get groupCheckTypeNone => '创建被限制,允许数目已满';
  @override
  String get groupCheckTypeSuspend => '账户被暂停使用';
  @override
  String get groupCheckTypeDeny => '没有权限在此创建群聊';
  @override
  String get members => '成员';
  @override
  String get groupRequireConsent => "需要管理员同意";
  @override
  String get domain => '分布式域名';
  @override
  String get domainIntro => '管理自己的公开身份';
  @override
  String get domainShowProvider => '展示所有服务商';
  @override
  String get domainShowName => '展示所有注册名';
  @override
  String get domainName => '用户名';
  @override
  String get domainProvider => '服务商';
  @override
  String get domainProviderAdress => '服务商网络地址';
  @override
  String get domainAddProvider => '添加新的服务商';
  @override
  String get domainSearch => '域名搜索';
}
