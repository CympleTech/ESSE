import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/device_info.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/show_pin.dart';
import 'package:esse/widgets/qr_scan.dart';
import 'package:esse/pages/home.dart';
import 'package:esse/account.dart';
import 'package:esse/global.dart';
import 'package:esse/rpc.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/device/provider.dart';
import 'package:esse/apps/chat/provider.dart';

class AccountRestorePage extends StatefulWidget {
  const AccountRestorePage({Key key}) : super(key: key);

  @override
  _AccountRestorePageState createState() => _AccountRestorePageState();
}

class _AccountRestorePageState extends State<AccountRestorePage> {
  bool _statusChecked = false;
  String _name;

  TextEditingController _addrController = new TextEditingController();
  FocusNode _addrFocus = new FocusNode();
  bool _addrOnline = false;
  bool _addrChecked = false;

  TextEditingController _wordController = new TextEditingController();
  FocusNode _wordFocus = new FocusNode();
  List<String> _mnemoicWords = [];
  bool _wordChecked = false;

  @override
  initState() {
    super.initState();
    _addrFocus.addListener(() {
      setState(() {});
    });
    _wordFocus.addListener(() {
      setState(() {});
    });
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    List<Widget> mnemonicWordWidgets = [];
    this._mnemoicWords.asMap().forEach((index, value) {
      mnemonicWordWidgets.add(Chip(
          avatar: CircleAvatar(
              backgroundColor: color.primary,
              child: Text("${index + 1}",
                  style: TextStyle(fontSize: 12, color: Colors.white))),
          label: Text(value.trim(), style: TextStyle(fontSize: 16)),
          backgroundColor: color.surface,
          padding: EdgeInsets.all(8.0),
          onDeleted: () => _deleteWord(index),
          deleteIconColor: Color(0xFFADB0BB)));
    });

    return Scaffold(
      body: SafeArea(
          child: Container(
              padding: const EdgeInsets.all(20.0),
              child: SingleChildScrollView(
                child: Column(
                  children: <Widget>[
                    Container(
                      width: 700.0,
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.start,
                        crossAxisAlignment: CrossAxisAlignment.center,
                        children: [
                          GestureDetector(
                            onTap: () => Navigator.of(context).pop(),
                            child: Container(
                              width: 40.0,
                              height: 40.0,
                              decoration: BoxDecoration(
                                color: color.primaryVariant,
                                borderRadius: BorderRadius.circular(15.0)),
                              child: Center(
                                child: Icon(Icons.arrow_back, color: Colors.white)),
                          )),
                          const SizedBox(width: 32.0),
                          Expanded(
                            child: Text(
                              lang.loginRestoreOnline,
                              style: TextStyle(
                                fontSize: 20.0,
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                          ),
                          TextButton(
                            onPressed: _scanQr,
                            child: Icon(Icons.qr_code_scanner_rounded)),
                    ])),
                    const SizedBox(height: 32.0),
                    Text(
                      'Online Network Address',
                      style:
                      TextStyle(fontSize: 18.0, fontWeight: FontWeight.bold),
                    ),
                    const SizedBox(height: 16.0),
                    Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: <Widget>[
                        Container(
                          height: 50.0,
                          width: 450.0,
                          child: Row(children: [
                              Expanded(
                                child: Container(
                                  padding: const EdgeInsets.symmetric(
                                    horizontal: 20.0),
                                  decoration: BoxDecoration(
                                    color: this._addrFocus.hasFocus
                                    ? color.primaryVariant
                                    : color.surface,
                                    border: Border.all(
                                      color: this._addrFocus.hasFocus
                                      ? color.primary
                                      : color.surface),
                                    borderRadius: BorderRadius.circular(15.0),
                                  ),
                                  child: TextField(
                                    style: TextStyle(
                                      color: Color(0xFF1C1939),
                                      fontSize: 16.0),
                                    decoration: InputDecoration(
                                      border: InputBorder.none,
                                      hintText: lang.address),
                                    controller: this._addrController,
                                    focusNode: this._addrFocus,
                                    onSubmitted: (_v) => _checkAddrOnline(),
                                    onChanged: (v) {
                                      if (v.length > 0 &&
                                        !this._addrChecked) {
                                        setState(() {
                                            this._addrChecked = true;
                                        });
                                      }
                                  }),
                                ),
                              ),
                              if (this._addrOnline)
                              Container(
                                padding: const EdgeInsets.only(left: 8.0),
                                child: Icon(Icons.cloud_done_rounded,
                                  color: Colors.green),
                              ),
                              const SizedBox(width: 8.0),
                              Container(
                                width: 100.0,
                                child: InkWell(
                                  onTap: this._addrChecked
                                  ? _checkAddrOnline
                                  : null,
                                  child: Container(
                                    height: 45.0,
                                    decoration: BoxDecoration(
                                      color: this._addrChecked
                                      ? color.primary
                                      : Color(0xFFADB0BB),
                                      borderRadius:
                                      BorderRadius.circular(15.0)),
                                    child: Center(
                                      child: Text(lang.search,
                                        style: TextStyle(
                                          fontSize: 16.0,
                                          color: Colors.white))),
                              ))),
                        ])),
                        const SizedBox(height: 32.0),
                        Text(
                          'Mnemoic Words',
                          style: TextStyle(
                            fontSize: 18.0, fontWeight: FontWeight.bold),
                        ),
                        const SizedBox(height: 16.0),
                        Container(
                          width: 450.0,
                          alignment: Alignment.center,
                          constraints: BoxConstraints(minHeight: 170.0),
                          padding: const EdgeInsets.all(10.0),
                          decoration: BoxDecoration(
                            border: Border.all(color: Color(0x40ADB0BB)),
                            borderRadius: BorderRadius.circular(15),
                          ),
                          child: Wrap(
                            spacing: 10.0,
                            runSpacing: 5.0,
                            alignment: WrapAlignment.center,
                            children: mnemonicWordWidgets,
                        )),
                        const SizedBox(height: 16.0),
                        Container(
                          height: 50.0,
                          width: 450.0,
                          child: Row(children: [
                              Container(
                                padding: const EdgeInsets.only(right: 8.0),
                                width: 40.0,
                                height: 40.0,
                                child: CircleAvatar(
                                  backgroundColor: this._wordFocus.hasFocus
                                  ? color.primary
                                  : color.surface,
                                  child: Text(
                                    (this._mnemoicWords.length + 1)
                                    .toString(),
                                    style: TextStyle(
                                      fontSize: 12,
                                      color: Colors.white))),
                              ),
                              Expanded(
                                child: Container(
                                  padding: const EdgeInsets.symmetric(
                                    horizontal: 20.0),
                                  decoration: BoxDecoration(
                                    color: this._wordFocus.hasFocus
                                    ? color.primaryVariant
                                    : color.surface,
                                    border: Border.all(
                                      color: this._wordFocus.hasFocus
                                      ? color.primary
                                      : color.surface),
                                    borderRadius: BorderRadius.circular(15.0),
                                  ),
                                  child: TextField(
                                    enabled: !this._statusChecked,
                                    style: TextStyle(fontSize: 16.0),
                                    decoration: InputDecoration(
                                      border: InputBorder.none,
                                      hintText: 'words'),
                                    controller: this._wordController,
                                    focusNode: this._wordFocus,
                                    onSubmitted: (_v) => _addWord(),
                                    onChanged: (v) {
                                      if (v.length > 0 &&
                                        !this._wordChecked) {
                                        setState(() {
                                            this._wordChecked = true;
                                        });
                                      }
                                  }),
                                ),
                              ),
                              const SizedBox(width: 8.0),
                              Container(
                                width: 100.0,
                                child: InkWell(
                                  onTap:
                                  this._wordChecked ? _addWord : null,
                                  child: Container(
                                    height: 45.0,
                                    decoration: BoxDecoration(
                                      color: this._wordChecked
                                      ? color.primary
                                      : Color(0xFFADB0BB),
                                      borderRadius:
                                      BorderRadius.circular(15.0)),
                                    child: Center(
                                      child: Text(lang.add,
                                        style: TextStyle(
                                          fontSize: 16.0,
                                          color: Colors.white))),
                              ))),
                        ])),
                        const SizedBox(height: 32.0),
                        ButtonText(
                          text: lang.next,
                          enable: _statusChecked,
                          action: () => _mnemonicRegister(lang.unknown, lang.setPin),
                        ),
                        _footer(
                          lang.hasAccount, () => Navigator.of(context).pop()),
                    ])
                ]),
          ))),
    );
  }

  _scanQr() {
    Navigator.push(context,
      MaterialPageRoute(
        builder: (context) => QRScan(callback: (isOk, app, params) {
            Navigator.of(context).pop();
            if (app == 'distribute' && params.length == 4) {
              final name = params[0];
              final id = params[1];
              final addr = params[2];
              final mnemonicWords = params[3];
              setState(() {
                  this._addrOnline = true;
                  this._addrChecked = false;
                  this._wordController.text = '';
                  this._wordChecked = false;
                  this._statusChecked = true;
                  this._name = name;
                  this._addrController.text = addr;
                  this._mnemoicWords = mnemonicWords.split(" ");
              });
            }
    })));
  }

  _checkAddrOnline() {
    FocusScope.of(context).unfocus();
    setState(() {
      this._addrOnline = true;
      this._addrChecked = false;
    });
  }

  _addWord() {
    final word = this._wordController.text.trim();
    if (word.length == 0) {
      return;
    }

    setState(() {
      this._mnemoicWords.add(word);
      if (this._mnemoicWords.length < 12) {
        this._wordController.text = '';
        this._wordFocus.requestFocus();
        this._wordChecked = false;
      } else {
        this._wordController.text = '';
        this._wordChecked = false;
        this._statusChecked = true;
      }
    });
  }

  _deleteWord(int index) {
    setState(() {
      this._mnemoicWords.removeAt(index);
      this._statusChecked = false;
    });
  }

  _mnemonicRegister(String defaultName, String title) async {
    print(this._mnemoicWords);
    if (this._mnemoicWords.length != 12) {
      return;
    }

    final mnemonic = this._mnemoicWords.join(' ');
    if (this._name == null) {
      this._name = defaultName;
    }
    var addr = this._addrController.text;
    if (addr.length > 2 && addr.substring(0, 2) == '0x') {
      //substring(2); if has 0x, need remove
      addr = addr.substring(2);
    }
    final info = await deviceInfo();

    showShadowDialog(
      context,
      Icons.security_rounded,
      title,
      SetPinWords(
        callback: (key, lock) async {
          Navigator.of(context).pop();
          // send to core node service by rpc.
          final res = await httpPost(Global.httpRpc, 'account-restore',
            [this._name, lock, mnemonic, addr, info[0], info[1]]);

          if (res.isOk) {
            // save this User
            final account = Account(res.params[0], this._name, lock);

            Provider.of<AccountProvider>(context, listen: false).addAccount(account);
            Provider.of<DeviceProvider>(context, listen: false).updateActived();
            Provider.of<ChatProvider>(context, listen: false).updateActived();

            Navigator.pushReplacement(context, MaterialPageRoute(builder: (_) => HomePage()));
          } else {
            // TODO tostor error
            print(res.error);
          }
      }),
      20.0,
    );
  }
}

Widget _footer(String text1, Function callback) {
  return Padding(
    padding: const EdgeInsets.only(top: 20),
    child: Center(
      child: TextButton(
        onPressed: callback,
        child: Text(
          text1,
          style: TextStyle(fontSize: 16),
        ),
      ),
    ),
  );
}
