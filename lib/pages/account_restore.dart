import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/show_pin.dart';
import 'package:esse/widgets/qr_scan.dart';
import 'package:esse/account.dart';
import 'package:esse/rpc.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/device/provider.dart';

class AccountRestorePage extends StatefulWidget {
  const AccountRestorePage({Key? key}) : super(key: key);

  @override
  _AccountRestorePageState createState() => _AccountRestorePageState();
}

class _AccountRestorePageState extends State<AccountRestorePage> {
  bool _statusChecked = false;
  String _name = '';

  Language _selectedLang = Language.English;
  TextEditingController _wordController = new TextEditingController();
  FocusNode _wordFocus = new FocusNode();
  List<String> _mnemoicWords = [];
  bool _wordChecked = false;
  bool _loading = false;

  @override
  initState() {
    super.initState();
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
              backgroundColor: Color(0xFF6174FF),
              child: Text("${index + 1}",
                  style: TextStyle(fontSize: 12, color: Colors.white))),
          label: Text(value.trim(), style: TextStyle(fontSize: 16)),
          backgroundColor: color.surface,
          padding: EdgeInsets.all(8.0),
          onDeleted: () => _deleteWord(index),
          deleteIconColor: Color(0xFFADB0BB)));
    });

    double maxHeight = (MediaQuery.of(context).size.height - 600) / 2;
    if (maxHeight < 20.0) {
      maxHeight = 20.0;
    }

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
                                color: Color(0xFF6174FF),
                                borderRadius: BorderRadius.circular(10.0)),
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
                    SizedBox(height: maxHeight),
                    Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: <Widget>[
                        Container(
                          width: 600.0,
                          height: 45.0,
                          padding: const EdgeInsets.only(left: 20, right: 10),
                          margin: const EdgeInsets.symmetric(vertical: 16.0),
                          decoration: BoxDecoration(
                            color: color.surface,
                            borderRadius: BorderRadius.circular(10.0)),
                          child: DropdownButtonHideUnderline(
                            child: Theme(
                              data: Theme.of(context).copyWith(
                                canvasColor: color.surface,
                              ),
                              child: DropdownButton<Language>(
                                hint: Text(lang.loginChooseAccount,
                                  style: TextStyle(fontSize: 16)),
                                iconEnabledColor: Color(0xFFADB0BB),
                                value: _selectedLang,
                                onChanged: (Language? m) {
                                  if (m != null) {
                                    setState(() {
                                        _selectedLang = m;
                                    });
                                  }
                                },
                                items: MNEMONIC_LANGUAGE.map((Language m) {
                                    return DropdownMenuItem<Language>(
                                      value: m,
                                      child: Text(m.localizations(context),
                                        style: TextStyle(fontSize: 16)));
                                }).toList(),
                            )),
                          ),
                        ),
                        Container(
                          width: 600.0,
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
                          width: 600.0,
                          child: Row(children: [
                              Container(
                                padding: const EdgeInsets.only(right: 8.0),
                                width: 40.0,
                                height: 40.0,
                                child: CircleAvatar(
                                  backgroundColor: this._wordFocus.hasFocus
                                  ? Color(0xFF6174FF)
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
                                    color: color.surface,
                                    border: Border.all(
                                      color: this._wordFocus.hasFocus
                                      ? color.primary
                                      : color.surface),
                                    borderRadius: BorderRadius.circular(10.0),
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
                                      ? Color(0xFF6174FF)
                                      : Color(0xFFADB0BB),
                                      borderRadius:
                                      BorderRadius.circular(10.0)),
                                    child: Center(
                                      child: Text(lang.add,
                                        style: TextStyle(
                                          fontSize: 16.0,
                                          color: Colors.white))),
                              ))),
                        ])),
                        const SizedBox(height: 32.0),
                        ButtonText(
                          text: this._loading ? lang.waiting : lang.next,
                          enable: _statusChecked && !this._loading,
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
            if (app == 'distribute' && params.length == 2) {
              final name = params[0];
              final mnemonicWords = params[1];
              setState(() {
                  this._wordController.text = '';
                  this._wordChecked = false;
                  this._statusChecked = true;
                  this._name = name;
                  this._mnemoicWords = mnemonicWords.split(" ");
              });
            }
    })));
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
    if (this._name == '') {
      this._name = defaultName;
    }

    showShadowDialog(
      context,
      Icons.security_rounded,
      title,
      SetPinWords(
        callback: (lock) async {
          Navigator.of(context).pop();
          setState(() { this._loading = false; });

          // send to core node service by rpc.
          final res = await httpPost('account-restore', [
              _selectedLang.toInt(), mnemonic, "", this._name, lock
          ]);

          if (res.isOk) {
            // save this User
            final account = Account(res.params[0], this._name, "", lock);

            Provider.of<AccountProvider>(context, listen: false).init(account);
            Provider.of<DeviceProvider>(context, listen: false).updateActived();

            Navigator.of(context).pushNamedAndRemoveUntil("/", (Route<dynamic> route) => false);
          } else {
            setState(() { this._loading = true; });
            // TODO tostor error
            print(res.error);
          }
      }),
      0.0
    );
  }
}

Widget _footer(String text1, VoidCallback callback) {
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
