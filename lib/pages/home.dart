import 'dart:ui' show ImageFilter;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:provider/provider.dart';
import 'package:flutter/cupertino.dart' show CupertinoSwitch;
import 'package:bottom_navy_bar/bottom_navy_bar.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/user_info.dart';
import 'package:esse/widgets/show_pin.dart';
import 'package:esse/widgets/qr_scan.dart';
import 'package:esse/pages/setting/profile.dart';
import 'package:esse/pages/setting/preference.dart';
import 'package:esse/pages/setting/network.dart';
import 'package:esse/pages/setting/about.dart';
import 'package:esse/account.dart';
import 'package:esse/global.dart';
import 'package:esse/options.dart';
import 'package:esse/provider.dart';
import 'package:esse/session.dart';

import 'package:esse/apps/device/provider.dart';
import 'package:esse/apps/device/page.dart';
import 'package:esse/apps/chat/list.dart';
import 'package:esse/apps/chat/detail.dart';
import 'package:esse/apps/chat/add.dart';
import 'package:esse/apps/file/models.dart';
import 'package:esse/apps/file/list.dart';
import 'package:esse/apps/service/models.dart';
import 'package:esse/apps/assistant/page.dart';
import 'package:esse/apps/group/detail.dart';

class HomePage extends StatelessWidget {
  //final Account account;
  const HomePage({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final isDesktop = isDisplayDesktop(context);
    final colorScheme = Theme.of(context).colorScheme;
    final isLight = colorScheme.brightness == Brightness.light;

    SystemUiOverlayStyle style = SystemUiOverlayStyle.light;
    if (isLight) {
      style = SystemUiOverlayStyle.dark;
    }

    return WillPopScope(
      onWillPop: () async {
        SystemChannels.platform.invokeMethod('SystemNavigator.pop');
        return false;
      },
      child: Scaffold(
        drawer: const DrawerWidget(),
        drawerScrimColor: const Color(0x26ADB0BB),
        body: AnnotatedRegion<SystemUiOverlayStyle>(
          value: style.copyWith(statusBarColor: colorScheme.secondary),
          child: SafeArea(
            child: isDesktop
            ? Row(children: [
                Container(
                  width: 320.0,
                  decoration: BoxDecoration(color: colorScheme.secondary),
                  child: HomeList()
                ),
                Expanded(child: context.watch<AccountProvider>().coreShowWidget),
            ])
            : HomeList()
    ))));
  }
}

class HomeList extends StatefulWidget {
  const HomeList({Key? key}) : super(key: key);

  @override
  _HomeListState createState() => _HomeListState();
}

class _HomeListState extends State<HomeList> {
  int _currentIndex = 0;
  PageController _pageController = PageController();

  @override
  void dispose() {
    _pageController.dispose();
    super.dispose();
  }

  _scanQr(bool isDesktop) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => QRScan(callback: (isOk, app, params) {
            Navigator.of(context).pop();
            if (app == 'add-friend' && params.length == 3) {
              final id = gidParse(params[0]);
              final addr = addrParse(params[1]);
              final name = params[2].trim();
              final widget = ChatAdd(id: id, addr: addr, name: name);
              Provider.of<AccountProvider>(context, listen: false).systemAppFriendAddNew = false;
              if (isDesktop) {
                Provider.of<AccountProvider>(context, listen: false).updateActivedWidget(widget);
              } else {
                Navigator.push(context, MaterialPageRoute(builder: (_) => widget));
              }
            } else if (app == 'distribute' && params.length == 4) {
              //final _name = params[0].trim();
              //final id = gidParse(params[1]);
              final addr = addrParse(params[2]);
              //final _mnemonicWords = params[3];
              Provider.of<DeviceProvider>(context, listen: false).connect(addr);
            }
    })));
  }

  @override
  Widget build(BuildContext context) {
    final isDesktop = isDisplayDesktop(context);
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final provider = context.watch<AccountProvider>();
    final allKeys = provider.topKeys + provider.orderKeys;
    final sessions = provider.sessions;

    return Scaffold(
      backgroundColor: Colors.transparent,
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.menu),
          onPressed: () => Scaffold.of(context).openDrawer(),
        ),
        bottom: isDesktop ? PreferredSize(
          child: Container(color: const Color(0x40ADB0BB), height: 1.0),
          preferredSize: Size.fromHeight(1.0)): null,
        actions: [
          IconButton(
            icon: const Icon(Icons.search),
            onPressed: null,
          ),
          Container(
            margin: const EdgeInsets.symmetric(horizontal: 20.0),
            alignment: Alignment.center,
            child: Stack(
              children: <Widget>[
                PopupMenuButton<int>(
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(15)),
                  color: const Color(0xFFEDEDED),
                  child: Icon(Icons.add_circle_outline, color: color.primary),
                  onSelected: (int value) {
                    if (value == 0) {
                      _scanQr(isDesktop);
                    } else if (value == 1) {
                      final widget = ChatAdd();
                      provider.systemAppFriendAddNew = false;
                      if (isDesktop) {
                        provider.updateActivedWidget(widget);
                      } else {
                        setState(() {});
                        Navigator.push(context,
                          MaterialPageRoute(builder: (_) => widget));
                      }
                    } else if (value == 2) {
                      // final widget = GroupAddPage();
                      // if (isDesktop) {
                      //   provider.updateActivedWidget(widget);
                      // } else {
                      //   setState(() {});
                      //   Navigator.push(context,
                      //     MaterialPageRoute(builder: (_) => widget));
                      // }
                    } else if (value == 3) {
                      showShadowDialog(
                        context,
                        Icons.info,
                        lang.info,
                        UserInfo(
                          app: 'add-friend',
                          id: provider.id,
                          name: provider.activedAccount.name,
                          addr: Global.addr));
                    }
                  },
                  itemBuilder: (context) {
                    return <PopupMenuEntry<int>>[
                      _menuItem(0, Icons.qr_code_scanner_rounded, lang.scan),
                      _menuItem(1, Icons.person_add_rounded, lang.addFriend,
                        provider.systemAppFriendAddNew),
                      _menuItem(
                        2, Icons.group_add_rounded, lang.groupChatAdd),
                      _menuItem(3, Icons.qr_code_rounded, lang.myQrcode),
                    ];
                  },
                ),
                if (provider.systemAppFriendAddNew)
                Positioned(
                  top: 0,
                  right: 0,
                  child: Container(
                    width: 8.0,
                    height: 8.0,
                    decoration: BoxDecoration(
                      color: Colors.red,
                      shape: BoxShape.circle,
                ))),
              ],
            ),
          )
      ]),
      body: SizedBox.expand(
        child: PageView(
          controller: _pageController,
          onPageChanged: (index) {
            setState(() => _currentIndex = index);
          },
          children: <Widget>[
            ListView.builder(
              itemCount: allKeys.length,
              itemBuilder: (BuildContext ctx, int index) =>
              _SessionWidget(session: sessions[allKeys[index]]!),
            ),
            ListView.builder(
              itemCount: HOME_DIRECTORY.length,
              itemBuilder: (BuildContext ctx, int index) {
                final params = HOME_DIRECTORY[index].params(lang);
                return ListTile(
                  leading: Icon(params[0], color: Color(0xFF6174FF)),
                  title: Text(params[1], style: TextStyle(fontSize: 16.0)),
                  trailing: Icon(Icons.keyboard_arrow_right),
                  onTap: () {
                    final widget = FilesList(path: params[2]);
                    if (widget != null) {
                      if (isDesktop) {
                        Provider.of<AccountProvider>(context, listen: false).updateActivedWidget(widget);
                      } else {
                        Navigator.push(context, MaterialPageRoute(builder: (_) => widget));
                      }
                    }
                  }
                );
              }
            ),
            ListView.builder(
              itemCount: INNER_SERVICES.length,
              itemBuilder: (BuildContext ctx, int index) {
                final params = INNER_SERVICES[index].params(lang);
                return ListTile(
                  leading: Container(
                    width: 40.0,
                    height: 40.0,
                    padding: const EdgeInsets.all(6.0),
                    decoration: BoxDecoration(
                      borderRadius: BorderRadius.circular(10.0),
                    ),
                    child: Image.asset(params[2]),
                  ),
                  title: Text(params[0], style: TextStyle(fontSize: 16.0)),
                  subtitle: Text(params[1], style: TextStyle(fontSize: 12.0)),
                  trailing: Icon(Icons.keyboard_arrow_right),
                  onTap: () {
                    final widget = INNER_SERVICES[index].callback();
                    if (widget != null) {
                      if (isDesktop) {
                        Provider.of<AccountProvider>(context, listen: false).updateActivedWidget(widget);
                      } else {
                        Navigator.push(context, MaterialPageRoute(builder: (_) => widget));
                      }
                    }
                  },
                );
              }
            ),
          ],
        ),
      ),
      bottomNavigationBar: BottomNavyBar(
        backgroundColor: color.secondary,
        selectedIndex: _currentIndex,
        showElevation: true,
        containerHeight: 50.0,
        onItemSelected: (index) => setState(() {
            _currentIndex = index;
            _pageController.animateToPage(index,
              duration: Duration(milliseconds: 300), curve: Curves.ease);
        }),
        items: [
          BottomNavyBarItem(
            icon: Icon(Icons.sms),
            title: Text(lang.sessions, style: TextStyle(fontSize: 15.0)),
            activeColor: Color(0xFF6174FF),
            inactiveColor: Colors.grey,
          ),
          BottomNavyBarItem(
            icon: Icon(Icons.source),
            title: Text(lang.dataCenter, style: TextStyle(fontSize: 15.0)),
            activeColor: Color(0xFF6174FF),
            inactiveColor: Colors.grey,
          ),
          BottomNavyBarItem(
            icon: Icon(Icons.apps),
            title: Text(lang.services, style: TextStyle(fontSize: 15.0)),
            activeColor: Color(0xFF6174FF),
            inactiveColor: Colors.grey,
          ),
        ],
      )
    );
  }
}

class DrawerWidget extends StatelessWidget {
  const DrawerWidget({Key? key}) : super(key: key);

  Widget _listAccount(context, Account account, Color color, lang) {
    return InkWell(
        onTap: account.online
            ? () {
                Navigator.of(context).pop();
                Provider.of<AccountProvider>(context, listen: false)
                    .updateActivedAccount(account.gid, account.pin);
                Provider.of<DeviceProvider>(context, listen: false)
                    .updateActived();
              }
            : null,
        child: Padding(
            padding:
                const EdgeInsets.symmetric(vertical: 5.0, horizontal: 10.0),
            child: Row(children: [
              account.showAvatar(online: account.online),
              const SizedBox(width: 10.0),
              Expanded(
                child: Text(account.name,
                    maxLines: 1, overflow: TextOverflow.ellipsis),
              ),
              const SizedBox(width: 10.0),
              Transform.scale(
                scale: 0.7,
                child: CupertinoSwitch(
                  activeColor: color,
                  value: account.online,
                  onChanged: (value) {
                    if (value) {
                      showShadowDialog(
                        context,
                        Icons.security_rounded,
                        lang.verifyPin,
                        PinWords(
                          gid: account.gid,
                          callback: (key) async {
                            Navigator.of(context).pop();
                            Provider.of<AccountProvider>(context,
                              listen: false)
                            .onlineAccount(account.gid, key);
                        }),
                        0.0,
                      );
                    } else {
                      Provider.of<AccountProvider>(context, listen: false)
                          .offlineAccount(account.gid);
                    }
                  },
                ),
              ),
            ])));
  }

  _showPage(Widget widget, bool isDesktop, context) {
    if (isDesktop) {
      Provider.of<AccountProvider>(context, listen: false)
          .updateActivedWidget(widget);
    } else {
      Navigator.push(context, MaterialPageRoute(builder: (_) => widget));
    }
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isLight = color.brightness == Brightness.light;
    final isDesktop = isDisplayDesktop(context);

    final provider = context.watch<AccountProvider>();
    final me = provider.activedAccount;
    final accounts = provider.accounts;

    List<Widget> accountsWidget = [];
    accounts.forEach((gid, account) {
      if (gid != me.gid) {
        accountsWidget.add(_listAccount(context, account, color.primary, lang));
      }
    });

    return Drawer(
        child: BackdropFilter(
            filter: ImageFilter.blur(sigmaX: 4.0, sigmaY: 4.0),
            child: SafeArea(
                child: Container(
              decoration: BoxDecoration(
                image: DecorationImage(
                  image: AssetImage(isLight
                      ? 'assets/images/background_light.jpg'
                      : 'assets/images/background_dark.jpg'),
                  fit: BoxFit.cover,
                ),
              ),
              child: ListView(
                padding: EdgeInsets.zero,
                children: <Widget>[
                  Padding(
                    padding: const EdgeInsets.symmetric(vertical: 20.0),
                    child: Center(
                        child: me.showAvatar(width: 100.0, needOnline: false)),
                  ),
                  Theme(
                    data: Theme.of(context)
                        .copyWith(dividerColor: Colors.transparent),
                    child: ExpansionTile(
                      title: Container(
                          padding: const EdgeInsets.only(left: 25.0),
                          alignment: Alignment.center,
                          child: Text(
                            "${me.name}",
                            style: TextStyle(
                                fontWeight: FontWeight.bold, fontSize: 16.0),
                          )),
                      children: accountsWidget,
                    ),
                  ),
                  const SizedBox(height: 5.0),
                  const Divider(height: 1.0, color: Color(0x40ADB0BB)),
                  ListTile(
                      leading: Icon(Icons.person, color: color.primary),
                      title: Text(lang.profile,
                          textAlign: TextAlign.left,
                          style: TextStyle(fontSize: 16.0)),
                      onTap: () {
                        Navigator.pop(context);
                        _showPage(ProfileDetail(), isDesktop, context);
                      }),
                  ListTile(
                      leading: Icon(Icons.devices_other_rounded,
                          color: color.primary),
                      title: Text(lang.devices,
                          textAlign: TextAlign.left,
                          style: TextStyle(fontSize: 16.0)),
                      onTap: () {
                        Navigator.pop(context);
                        _showPage(DevicesPage(), isDesktop, context);
                      }),
                  ListTile(
                      leading: Icon(Icons.account_tree, color: color.primary),
                      title: Text(lang.network,
                          textAlign: TextAlign.left,
                          style: TextStyle(fontSize: 16.0)),
                      onTap: () {
                        Navigator.pop(context);
                        _showPage(NetworkDetail(), isDesktop, context);
                      }),
                  ListTile(
                      leading: Icon(Icons.language, color: color.primary),
                      title: Text(lang.preference,
                          textAlign: TextAlign.left,
                          style: TextStyle(fontSize: 16.0)),
                      onTap: () {
                        Navigator.pop(context);
                        showShadowDialog(context, Icons.language,
                            lang.preference, PreferenceDetail());
                      }),
                  ListTile(
                      leading: Icon(Icons.info, color: color.primary),
                      title: Text(lang.aboutUs,
                          textAlign: TextAlign.left,
                          style: TextStyle(fontSize: 16.0)),
                      onTap: () {
                        Navigator.pop(context);
                        showShadowDialog(
                            context, Icons.info, lang.aboutUs, AboutDetail());
                      }),
                  ListTile(
                      leading: Icon(Icons.brightness_2, color: color.primary),
                      title: Text(lang.nightly,
                          textAlign: TextAlign.left,
                          style: TextStyle(fontSize: 16.0)),
                      trailing: Transform.scale(
                        scale: 0.7,
                        child: CupertinoSwitch(
                          activeColor: Color(0xFF6174FF),
                          value: !isLight,
                          onChanged: (_) {
                            final themeMode =
                                isLight ? ThemeMode.dark : ThemeMode.light;
                            context.read<Options>().changeTheme(themeMode);
                          },
                        ),
                      )),
                  ListTile(
                      leading: Icon(Icons.logout, color: color.primary),
                      title: Text(lang.logout,
                          textAlign: TextAlign.left,
                          style: TextStyle(fontSize: 16.0)),
                      onTap: () {
                        context.read<AccountProvider>().logout();
                        context.read<DeviceProvider>().clear();
                        Navigator.of(context).pushNamedAndRemoveUntil(
                            "/security", (Route<dynamic> route) => false);
                      }),
                  SizedBox(height: 20.0),
                ],
              ),
            ))));
  }
}

class _SessionWidget extends StatelessWidget {
  final Session session;
  const _SessionWidget({Key? key, required this.session}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);
    final params = session.parse(lang);

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () {
        Widget? coreWidget;

        switch (session.type) {
          case SessionType.Chat:
            coreWidget = ChatDetail(id: session.fid);
            break;
          case SessionType.Group:
            coreWidget = GroupChatDetail(id: session.fid);
            break;
          case SessionType.Assistant:
            coreWidget = AssistantDetail();
            break;
          default:
            break; // TODO
        }

        context.read<AccountProvider>().updateActivedSession(session.id);

        if (coreWidget != null) {
          if (!isDesktop) {
            Navigator.push(
                context, MaterialPageRoute(builder: (_) => coreWidget!));
          } else {
            context.read<AccountProvider>().updateActivedWidget(coreWidget);
          }
        }
      },
      child: Container(
        height: 55.0,
        child: Row(
          children: [
            Container(
              width: 45.0,
              height: 45.0,
              margin: const EdgeInsets.only(left: 10.0, right: 12.0),
              child: params[0],
            ),
            Expanded(
              child: Container(
                height: 55.0,
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Expanded(
                              child: Text(params[1],
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                  style: TextStyle(fontSize: 16.0))),
                          Container(
                            margin:
                                const EdgeInsets.only(left: 15.0, right: 20.0),
                            child: Text(params[3],
                                style: const TextStyle(
                                    color: Color(0xFFADB0BB), fontSize: 12.0)),
                          )
                        ]),
                    const SizedBox(height: 4.0),
                    Row(children: [
                      Expanded(
                        child: Row(
                            crossAxisAlignment: CrossAxisAlignment.center,
                            children: [
                              if (params[4] != null)
                                Container(
                                  margin: const EdgeInsets.only(right: 6.0),
                                  child: Icon(params[4],
                                      size: 16.0, color: Color(0xFFADB0BB)),
                                ),
                              Expanded(
                                child: Text(params[2],
                                    maxLines: 1,
                                    overflow: TextOverflow.ellipsis,
                                    style: const TextStyle(
                                        color: Color(0xFFADB0BB),
                                        fontSize: 12.0)),
                              )
                            ]),
                      ),
                      session.isClose
                          ? Container(
                              margin: const EdgeInsets.only(
                                  left: 15.0, right: 20.0),
                              child: Icon(Icons.block_rounded,
                                  color: Color(0xFFADB0BB), size: 14.0))
                          : Container(
                              width: 8.0,
                              height: 8.0,
                              margin: const EdgeInsets.only(
                                  left: 15.0, right: 20.0),
                              decoration: BoxDecoration(
                                  color: session.lastReaded
                                      ? color.background
                                      : Colors.red,
                                  shape: BoxShape.circle),
                            ),
                    ]),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

PopupMenuEntry<int> _menuItem(int value, IconData icon, String text,
    [bool hasNew = false]) {
  return PopupMenuItem<int>(
    value: value,
    child: Row(children: [
      Stack(
        children: <Widget>[
          Container(
            width: 30.0,
            height: 30.0,
            child: Icon(icon, color: Color(0xFF6174FF)),
          ),
          if (hasNew)
            Positioned(
                top: 0,
                right: 0,
                child: Container(
                    width: 8.0,
                    height: 8.0,
                    decoration: BoxDecoration(
                      color: Colors.red,
                      shape: BoxShape.circle,
                    ))),
        ],
      ),
      const SizedBox(width: 10.0),
      Text(text, style: TextStyle(color: Colors.black, fontSize: 16.0)),
    ]),
  );
}
