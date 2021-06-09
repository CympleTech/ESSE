import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/show_pin.dart';
import 'package:esse/pages/account_generate.dart';
import 'package:esse/pages/account_restore.dart';
import 'package:esse/utils/logined_cache.dart';
import 'package:esse/account.dart';
import 'package:esse/global.dart';
import 'package:esse/options.dart';
import 'package:esse/rpc.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/device/provider.dart';
import 'package:esse/apps/chat/provider.dart';
import 'package:esse/apps/group_chat/provider.dart';

class SecurityPage extends StatefulWidget {
  const SecurityPage({Key key}) : super(key: key);

  @override
  _SecurityPageState createState() => _SecurityPageState();
}

class _SecurityPageState extends State<SecurityPage> {
  Map<String, Account> _accounts = {};
  bool _accountsLoaded = false;

  String _selectedUserId;
  String _selectedUserLock;

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
          child: Container(
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
                    width: 100.0,
                    height: 100.0,
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
                  Text('ESSE', style: TextStyle(fontSize: 20.0)),
                  const SizedBox(height: 80.0),
                  loginForm(color, lang),
                  const SizedBox(height: 20.0),
                  ButtonText(text: lang.ok, enable: _accountsLoaded,
                    action: () => loginAction(lang.verifyPin)),
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
                              lang.loginRestore,
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
                              lang.loginNew,
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
          )
        )
      )
    );
  }

  void loadAccounts() async {
    // init rpc.
    if (!rpc.isLinked()) {
      await rpc.init(Global.wsRpc);
    }

    // check if has logined.
    final loginedAccounts = await getLogined();

    if (loginedAccounts.length != 0) {
      print("INFO: START LOGINED USE CACHE");
      final mainAccount = loginedAccounts[0];
      final res = await httpPost(Global.httpRpc, 'account-login', [mainAccount.gid, mainAccount.lock]);

      if (res.isOk) {
        Map<String, Account> accounts = {};
        loginedAccounts.forEach((account) {
            accounts[account.gid] = account;
        });

        Provider.of<AccountProvider>(context, listen: false).autoAccounts(mainAccount.gid, accounts);
        Provider.of<DeviceProvider>(context, listen: false).updateActived();
        Provider.of<ChatProvider>(context, listen: false).updateActived();
        Provider.of<GroupChatProvider>(context, listen: false).updateActived();

        Navigator.of(context).pushNamedAndRemoveUntil('/', ModalRoute.withName('/'));
        return;
      } else {
        // TODO tostor error
        print(res.error);
      }
    }

    print("INFO: START LOGINED WITH ACCOUNTS");
    final res = await httpPost(Global.httpRpc, 'account-list', []);
    if (res.isOk) {
      this._accounts.clear();
      res.params.forEach((param) {
          this._accounts[param[0]] = Account(param[0], param[1], param[2], param[3]);
      });
      Provider.of<AccountProvider>(context, listen: false).initAccounts(this._accounts);

      if (this._accounts.length > 0) {
        final accountId = this._accounts.keys.first;
        this._selectedUserId = this._accounts[accountId].gid;
        this._selectedUserLock = this._accounts[accountId].lock;
        this._accountsLoaded = true;
      }

      setState(() {});
    } else {
      // TODO tostor error
      print(res.error);
    }
  }

  void loginAction(String title) {
    showShadowDialog(
      context,
      Icons.security_rounded,
      title,
      PinWords(
        hashPin: this._selectedUserLock,
        callback: (pinWords, lock) async {
          Navigator.of(context).pop();
          final res = await httpPost(Global.httpRpc, 'account-login',
            [this._selectedUserId, lock]);

          if (res.isOk) {
            final mainAccount = this._accounts[this._selectedUserId];

            Provider.of<AccountProvider>(context, listen: false).updateActivedAccount(mainAccount.gid);
            Provider.of<DeviceProvider>(context, listen: false).updateActived();
            Provider.of<ChatProvider>(context, listen: false).updateActived();
            Provider.of<GroupChatProvider>(context, listen: false).updateActived();

            Navigator.of(context).pushNamedAndRemoveUntil('/', ModalRoute.withName('/'));
          } else {
            // TODO tostor error
            print(res.error);
          }
    }));
  }

  Widget loginForm(ColorScheme color, AppLocalizations lang) {
    return Container(
      width: 450.0,
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
            onChanged: (String gid) {
              setState(() {
                  this._selectedUserId = gid;
                  this._selectedUserLock = this._accounts[gid].lock;
              });
            },
            items: this._accounts.values.map((Account account) {
                return DropdownMenuItem<String>(
                  value: account.gid,
                  child: Row(
                    children: [
                      Expanded(
                        child: Text("${account.name}",
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(fontSize: 16)
                        ),
                      ),
                      Text(" (${account.printShortId()})", style: TextStyle(fontSize: 16)),
                    ]
                  ),
                );
            }).toList(),
          ),
        ),
    ));
  }
}
