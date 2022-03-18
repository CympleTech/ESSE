import 'dart:ui' show ImageFilter;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/show_pin.dart';
import 'package:esse/pages/account_generate.dart';
import 'package:esse/pages/account_restore.dart';
import 'package:esse/pages/account_quick.dart';
import 'package:esse/utils/logined_cache.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/utils/toast.dart';
import 'package:esse/account.dart';
import 'package:esse/global.dart';
import 'package:esse/rpc.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/device/provider.dart';

class SecurityPage extends StatefulWidget {
  const SecurityPage({Key? key}) : super(key: key);

  @override
  _SecurityPageState createState() => _SecurityPageState();
}

class _SecurityPageState extends State<SecurityPage> {
  Map<String, Account> _accounts = {};
  bool _loaded = false;
  bool _accountsLoaded = false;
  bool _loading = false;

  String _selectedUserId = '';

  SystemUiOverlayStyle style = SystemUiOverlayStyle.dark;

  @override
  initState() {
    super.initState();
    loadAccounts();
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final isLight = color.brightness == Brightness.light;
    final lang = AppLocalizations.of(context);

    if (isLight) {
      style = SystemUiOverlayStyle.dark;
    } else {
      style = SystemUiOverlayStyle.light;
    }

    double maxHeight = (MediaQuery.of(context).size.height - 400) / 2;
    if (maxHeight < 20.0) {
      maxHeight = 20.0;
    }

    return Scaffold(
      body: AnnotatedRegion<SystemUiOverlayStyle>(
        value: style.copyWith(statusBarColor: color.background),
        child: SafeArea(
          child: Stack(children: [
              Container(
                padding: const EdgeInsets.all(20.0),
                height: MediaQuery.of(context).size.height,
                decoration: BoxDecoration(
                  image: DecorationImage(
                    image: AssetImage(
                      isLight
                      ? 'assets/images/background_light.jpg'
                      : 'assets/images/background_dark.jpg'
                    ),
                    fit: BoxFit.cover,
                  ),
                ),
                child: SingleChildScrollView(
                  child: Column(
                    children: <Widget>[
                      SizedBox(height: maxHeight),
                      Container(
                        width: 120.0,
                        height: 120.0,
                        decoration: BoxDecoration(
                          boxShadow: [
                            BoxShadow(
                              color: Color(0xFF2B2E38).withOpacity(0.3),
                              spreadRadius: 5.0,
                              blurRadius: 15.0,
                              offset: Offset(0, 10),
                            ),
                          ],
                        ),
                        child: ClipRRect(
                          borderRadius: BorderRadius.circular(15.0),
                          child: Image(image: isLight
                            ? AssetImage('assets/logo/logo_light.png')
                            : AssetImage('assets/logo/logo_dark.png'))
                        )
                      ),
                      const SizedBox(height: 40.0),
                      Text('ESSE', style: TextStyle(fontSize: 20.0, fontWeight: FontWeight.bold)),
                      const SizedBox(height: 40.0),
                      loginForm(color, lang),
                      const SizedBox(height: 20.0),
                      ButtonText(text: this._loading ? lang.waiting : lang.ok,
                        enable: _accountsLoaded && !this._loading,
                        action: () => loginAction(lang.verifyPin, color, lang)),
                      const SizedBox(height: 20.0),
                      InkWell(
                        child: Container(width: 600.0, height: 50.0,
                          decoration: BoxDecoration(
                            border: Border.all(color: Color(0xFF6174FF)),
                            borderRadius: BorderRadius.circular(10.0)),
                          child: Center(child: Text(lang.loginQuick, style: TextStyle(
                                fontSize: 20.0, color: Color(0xFF6174FF)
                          ))),
                        ),
                        onTap: () => Navigator.push(context,
                          MaterialPageRoute(builder: (_) => AccountQuickPage())
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.only(top: 20),
                        child: Container(
                          child: Row(
                            mainAxisAlignment: MainAxisAlignment.center,
                            children: <Widget>[
                              TextButton(
                                onPressed: () => Navigator.push(context,
                                  MaterialPageRoute(builder: (_) => AccountRestorePage())),
                                child: Text(
                                  lang.importAccount,
                                  style: TextStyle(fontSize: 16),
                                ),
                              ),
                              const SizedBox(width: 10.0),
                              Text("|", style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
                              const SizedBox(width: 10.0),
                              TextButton(
                                onPressed: () => Navigator.push(context,
                                  MaterialPageRoute(builder: (_) => AccountGeneratePage())),
                                child: Text(
                                  lang.createAccount,
                                  style: TextStyle(fontSize: 16),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
                    ]
                  )
                )
              ),
              this._loaded ? Container() : LoaderTransparent(color: color.primary)
            ]
          )
        )
      )
    );
  }

  _handleLogined(Account account) {
    Provider.of<AccountProvider>(context, listen: false).init(account);
    Provider.of<DeviceProvider>(context, listen: false).updateActived();
    Navigator.of(context).pushNamedAndRemoveUntil("/", (Route<dynamic> route) => false);
  }

  void loadAccounts() async {
    // init rpc.
    if (!rpc.isLinked()) {
      await rpc.init(Global.wsRpc);
    }

    // check if has logined.
    final loginedAccount = await getLogined();
    if (loginedAccount != null) {
      print("INFO: START LOGINED USE CACHE");
      final res = await httpPost('account-login', [loginedAccount.pid, ""]);
      if (res.isOk) {
        _handleLogined(loginedAccount);
        return;
      } else {
        showShadowDialog(
          context,
          Icons.security_rounded,
          "PIN",
          PinWords(
            pid: loginedAccount.pid,
            callback: (key) async {
              Navigator.of(context).pop();
              loginedAccount.pin = key;
              _handleLogined(loginedAccount);
              return;
          }),
          0.0
        );
      }
    }

    print("INFO: START LOGINED WITH ACCOUNTS");
    final res = await httpPost('account-list', []);
    if (res.isOk) {
      this._accounts.clear();
      res.params.forEach((param) {
          this._accounts[param[0]] = Account(param[0], param[1], param[2]);
      });

      if (this._accounts.length > 0) {
        final accountId = this._accounts.keys.first;
        this._selectedUserId = this._accounts[accountId]!.pid;
        this._accountsLoaded = true;
      }
    } else {
      toast(context, res.error);
    }

    setState(() {
        this._loaded = true;
    });
  }

  void _verifyAfter(String lock) async {
    setState(() { this._loading = true; });
    final res = await httpPost('account-login', [this._selectedUserId, lock]);
    if (res.isOk) {
      Account account = this._accounts[this._selectedUserId]!;
      account.pin = lock;
      _handleLogined(account);
    } else {
      setState(() { this._loading = false; });
      toast(context, res.error);
    }
  }

  void loginAction(String title, color, lang) {
    showShadowDialog(
      context,
      Icons.security_rounded,
      title,
      PinWords(
        pid: this._selectedUserId,
        callback: (pinWords) async {
          Navigator.of(context).pop();
          _verifyAfter(pinWords);
      }),
      0.0,
    );
  }

  Widget loginForm(ColorScheme color, AppLocalizations lang) {
    return Container(
      width: 600.0,
      height: 50.0,
      padding: EdgeInsets.only(left: 20, right: 20),
      decoration: BoxDecoration(
        color: color.surface, borderRadius: BorderRadius.circular(15.0)),
      child: DropdownButtonHideUnderline(
        child: Theme(
          data: Theme.of(context).copyWith(
            canvasColor: color.surface,
          ),
          child: DropdownButton<String>(
            hint: Text(lang.loginChooseAccount, style: TextStyle(fontSize: 16)),
            iconEnabledColor: Color(0xFFADB0BB),
            isExpanded: true,
            value: this._selectedUserId,
            onChanged: (String? pid) {
              if (pid != null) {
                setState(() {
                    this._selectedUserId = pid;
                  });
              }
            },
            items: this._accounts.values.map((Account account) {
                return DropdownMenuItem<String>(
                  value: account.pid,
                  child: Row(
                    children: [
                      Expanded(
                        child: Text("${account.name}",
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(fontSize: 16)
                        ),
                      ),
                      Text(" (${pidPrint(account.pid)})", style: TextStyle(fontSize: 16)),
                    ]
                  ),
                );
            }).toList(),
          ),
        ),
    ));
  }
}

class LoaderTransparent extends StatelessWidget {
  final Color color;
  LoaderTransparent({required this.color});

  @override
  Widget build(BuildContext context) {
    final height = MediaQuery.of(context).size.height;
    final width = MediaQuery.of(context).size.width;

    return BackdropFilter(
      filter: ImageFilter.blur(sigmaX: 6.0, sigmaY: 6.0),
      child: Container(
        height: height,
        width: width,
        child: Center(
          child: SizedBox(
            height: 60.0,
            width: 60.0,
            child: CircularProgressIndicator(
              valueColor: AlwaysStoppedAnimation(this.color),
              strokeWidth: 12.0
            )
          )
        )
    ));
  }
}
