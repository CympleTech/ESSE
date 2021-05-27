import 'dart:async';
import 'dart:ui' show ImageFilter;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:provider/provider.dart';
import 'package:flutter/cupertino.dart' show CupertinoSwitch;

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/adaptive.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/user_info.dart';
import 'package:esse/widgets/list_system_app.dart';
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

import 'package:esse/apps/device/provider.dart';
import 'package:esse/apps/device/page.dart';
import 'package:esse/apps/chat/provider.dart';
import 'package:esse/apps/chat/list.dart';
import 'package:esse/apps/chat/add.dart';
import 'package:esse/apps/file/page.dart';
import 'package:esse/apps/service/list.dart';
import 'package:esse/apps/service/add.dart';
import 'package:esse/apps/assistant/page.dart';

class HomePage extends StatelessWidget {
  static GlobalKey<ScaffoldState> _scaffoldKey = new GlobalKey<ScaffoldState>();

  //final Account account;
  const HomePage({Key key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final isDesktop = isDisplayDesktop(context);
    final colorScheme = Theme.of(context).colorScheme;
    final isLight = colorScheme.brightness == Brightness.light;

    if (isDesktop) {
      return WillPopScope(
          onWillPop: () =>
              SystemChannels.platform.invokeMethod('SystemNavigator.pop'),
          child: Scaffold(
            key: _scaffoldKey,
            drawer: const DrawerWidget(),
            drawerScrimColor: Color(0x26ADB0BB),
            body: SafeArea(
                child: Row(children: [
              Container(
                width: 375.0,
                child: HomeList(_scaffoldKey),
              ),
              SizedBox(width: 20.0),
              Expanded(child: context.watch<AccountProvider>().coreShowWidget),
            ])),
          ));
    } else {
      var style;
      if (isLight) {
        style = SystemUiOverlayStyle.dark;
      } else {
        style = SystemUiOverlayStyle.light;
      }

      return WillPopScope(
          onWillPop: () =>
              SystemChannels.platform.invokeMethod('SystemNavigator.pop'),
          child: Scaffold(
            key: _scaffoldKey,
            drawer: const DrawerWidget(),
            drawerScrimColor: Color(0x26ADB0BB),
            body: AnnotatedRegion<SystemUiOverlayStyle>(
                value: style.copyWith(statusBarColor: colorScheme.background),
                child: SafeArea(
                  child: HomeList(_scaffoldKey),
                )),
          ));
    }
  }
}

class HomeList extends StatefulWidget {
  final GlobalKey<ScaffoldState> _scaffoldKey;
  HomeList(this._scaffoldKey);

  @override
  _HomeListState createState() => _HomeListState();
}

class _HomeListState extends State<HomeList> {
  _scanQr(bool isDesktop) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => QRScan(callback: (isOk, app, params) {
            Navigator.of(context).pop();
            if (app == 'add-friend' && params.length == 3) {
              final id = params[0];
              final addr = params[1];
              final name = params[2];
              final widget = ChatAddPage(id: id, addr: addr, name: name);
              Provider.of<AccountProvider>(context, listen: false)
              .systemAppFriendAddNew = false;
              if (isDesktop) {
                Provider.of<AccountProvider>(context, listen: false)
                .updateActivedWidget(widget);
              } else {
                Navigator.push(
                  context, MaterialPageRoute(builder: (_) => widget));
              }
            } else if (app == 'distribute' && params.length == 4) {
              final _name = params[0];
              final id = params[1];
              final addr = params[2];
              final _mnemonicWords = params[3];
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

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 10.0),
      child: Column(
        children: [
          Container(
            height: 30.0,
            padding: const EdgeInsets.symmetric(horizontal: 20.0),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                GestureDetector(
                  onTap: widget._scaffoldKey.currentState.openDrawer,
                  child: Icon(
                    Icons.menu_rounded,
                    color: color.primary,
                    size: 28.0,
                  ),
                ),
                Expanded(
                  child: Center(
                    child: Text(provider.homeShowTitle, style: TextStyle(fontWeight: FontWeight.bold)),
                )),
                Icon(Icons.search_rounded, color: color.primary),
                const SizedBox(width: 20.0),
                Stack(
                  children: <Widget>[
                    Container(
                      width: 28.0,
                      height: 28.0,
                      child: provider.homeShowTitle == ''
                      ? PopupMenuButton<int>(
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(15)),
                        color: const Color(0xFFEDEDED),
                        child: Icon(Icons.add_circle_outline_rounded,
                          color: color.primary),
                        onSelected: (int value) {
                          if (value == 0) {
                            _scanQr(isDesktop);
                          } else if (value == 1) {
                            final widget = ChatAddPage();
                            if (isDesktop) {
                              provider.updateActivedWidget(widget);
                            } else {
                              provider.systemAppFriendAddNew = false;
                              setState(() {});
                              Navigator.push(context, MaterialPageRoute(builder: (_) => widget));
                            }
                          } else if (value == 2) {
                            showShadowDialog(
                              context,
                              Icons.info,
                              lang.info,
                              UserInfo(
                                id: provider.activedAccount.id,
                                name: provider.activedAccount.name,
                                addr: Global.addr)
                            );
                          }
                        },
                        itemBuilder: (context) {
                          return <PopupMenuEntry<int>>[
                            _menuItem(0, Icons.qr_code_scanner_rounded, lang.scan),
                            _menuItem(1, Icons.person_add_rounded, lang.addFriend,
                              provider.systemAppFriendAddNew),
                            _menuItem(2, Icons.qr_code_rounded, lang.myQrcode),
                          ];
                        },
                      )
                      : GestureDetector(onTap: () => provider.updateToHome(),
                        child: Icon(Icons.home_outlined, color: color.primary))),
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
              ],
            ),
          ),
          const SizedBox(height: 10.0),
          const Divider(height: 1.0, color: Color(0x40ADB0BB)),
          const SizedBox(height: 5.0),
          Expanded(
            child: provider.homeShowWidget,
          )
        ],
      ),
    );
  }
}

class DrawerWidget extends StatelessWidget {
  const DrawerWidget({Key key}) : super(key: key);

  Widget _listAccount(context, Account account, Color color, lang) {
    return InkWell(
        onTap: account.online
            ? () {
                Navigator.of(context).pop();
                Provider.of<AccountProvider>(context, listen: false).updateActivedAccount(account.gid);
                Provider.of<DeviceProvider>(context, listen: false).updateActived();
                Provider.of<ChatProvider>(context, listen: false).updateActived();
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
                              hashPin: account.lock,
                              callback: (key, hash) async {
                                Navigator.of(context).pop();
                                Provider.of<AccountProvider>(context,
                                        listen: false)
                                    .onlineAccount(account.gid, hash);
                              }));
                    } else {
                      Provider.of<AccountProvider>(context, listen: false)
                          .offlineAccount(account.gid);
                    }
                  },
                ),
              ),
            ])));
  }

  _showDevices(context, bool isDesktop) {
    final widget = DevicesPage();
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
                    leading: Icon(Icons.people_rounded, color: color.primary),
                    title: Text(lang.contact, textAlign: TextAlign.left,
                      style: TextStyle(fontSize: 16.0)),
                    onTap: () {
                      Navigator.pop(context);
                      Provider.of<AccountProvider>(context, listen: false).updateActivedWidget(
                        null, lang.contact, ChatList()
                      );
                  }),
                  ListTile(
                    leading: Icon(Icons.grid_view_rounded, color: color.primary),
                    title: Text(lang.groups, textAlign: TextAlign.left,
                      style: TextStyle(fontSize: 16.0)),
                    onTap: () {
                      Navigator.pop(context);
                      Provider.of<AccountProvider>(context, listen: false).updateActivedWidget(
                        null, lang.groups, ServiceList()
                      );
                  }),
                  const Divider(height: 1.0, color: Color(0x40ADB0BB)),
                  ListTile(
                    leading: Icon(Icons.person, color: color.primary),
                    title: Text(lang.profile,
                      textAlign: TextAlign.left,
                      style: TextStyle(fontSize: 16.0)),
                    onTap: () {
                      Navigator.pop(context);
                      showShadowDialog(context, Icons.person, lang.profile,
                        ProfileDetail());
                  }),
                  ListTile(
                    leading: Icon(Icons.devices_other_rounded,
                      color: color.primary),
                    title: Text(lang.devices,
                      textAlign: TextAlign.left,
                      style: TextStyle(fontSize: 16.0)),
                    onTap: () {
                      Navigator.pop(context);
                      _showDevices(context, isDesktop);
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
                      leading: Icon(Icons.account_tree, color: color.primary),
                      title: Text(lang.network,
                          textAlign: TextAlign.left,
                          style: TextStyle(fontSize: 16.0)),
                      onTap: () {
                        Navigator.pop(context);
                        showShadowDialog(context, Icons.account_tree,
                            lang.network, NetworkDetail());
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
                        context.read<ChatProvider>().clear();
                        Navigator.of(context).pushReplacementNamed('/');
                      }),
                  SizedBox(height: 20.0),
                ],
              ),
            ))));
  }
}

Widget _menuItem(int value, IconData icon, String text,
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
