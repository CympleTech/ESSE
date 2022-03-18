import 'dart:convert' show base64;
import 'dart:typed_data' show Uint8List;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/show_pin.dart';
import 'package:esse/widgets/select_avatar.dart';
import 'package:esse/account.dart';
import 'package:esse/rpc.dart';
import 'package:esse/provider.dart';

import 'package:esse/pages/account_domain.dart';
import 'package:esse/apps/device/provider.dart';

class AccountGeneratePage extends StatefulWidget {
  const AccountGeneratePage({Key? key}) : super(key: key);

  @override
  _AccountGeneratePageState createState() => _AccountGeneratePageState();
}

class _AccountGeneratePageState extends State<AccountGeneratePage> {
  Language _selectedLang = Language.English;
  String _mnemoicWords = "";

  bool _mnemonicChecked = false;
  bool _registerChecked = false;
  bool _isAccount = false;
  bool _loading = false;

  TextEditingController _nameController = new TextEditingController();
  FocusNode _nameFocus = new FocusNode();

  Uint8List? _imageBytes;

  @override
  initState() {
    super.initState();

    _nameFocus.addListener(() {
        setState(() {});
    });
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Scaffold(
      body: SafeArea(
        child: Container(
          padding: const EdgeInsets.all(20.0),
          child: SingleChildScrollView(
            child: _isAccount ? _accountState(color, lang) : _mnemonicState(color, lang),
          )
        )
      ),
    );
  }

  void genMnemonic() async {
    final res = await httpPost('account-generate', [_selectedLang.toInt()]);
    if (res.isOk) {
      this._mnemoicWords = res.params[0];
      this._mnemonicChecked = true;
      setState(() {});
    } else {
      // TODO tostor error
      print(res.error);
    }
  }

  void registerNewAction(String title) async {
    final mnemonic = _mnemoicWords;
    final name = _nameController.text;
    final avatar = _imageBytes != null ? base64.encode(_imageBytes!) : "";

    if (!_registerChecked) {
      return;
    }

    showShadowDialog(
      context,
      Icons.security_rounded,
      title,
      SetPinWords(
        callback: (lock) async {
          Navigator.of(context).pop();
          setState(() { this._loading = true; });

          // send to core node service by rpc.
          final res = await httpPost('account-create', [
              _selectedLang.toInt(), mnemonic, "", name, lock, avatar
          ]);

          if (res.isOk) {
            final pid = res.params[0];
            final login = await httpPost('account-login', [pid, lock]);

            if (login.isOk) {
              // save this User
              final account = Account(pid, name, avatar, lock);

              Provider.of<AccountProvider>(context, listen: false).init(account);
              Provider.of<DeviceProvider>(context, listen: false).updateActived();

              Navigator.push(context, MaterialPageRoute(builder: (_) => AccountDomainScreen(
                    name: name,
              )));
            } else {
              setState(() { this._loading = false; });
            }
          } else {
            setState(() { this._loading = false; });
            // TODO tostor error
            print(res.error);
          }
      }),
      0.0
    );
  }

  Widget _mnemonicState(ColorScheme color, AppLocalizations lang) {
    List<Widget> mnemonicWordWidgets = [];
    if (this._mnemoicWords.length > 1) {
      this._mnemoicWords.split(" ").asMap().forEach((index, value) {
          mnemonicWordWidgets.add(Chip(
              avatar: CircleAvatar(
                backgroundColor: Color(0xFF6174FF),
                child: Text("${index + 1}",
                  style: TextStyle(fontSize: 12, color: Colors.white))),
              label: Text(value.trim(), style: TextStyle(fontSize: 16)),
              backgroundColor: color.surface,
              padding: EdgeInsets.all(8.0),
          ));
      });
    }

    double maxHeight = (MediaQuery.of(context).size.height - 500) / 2;
    if (maxHeight < 20.0) {
      maxHeight = 20.0;
    }

    return Column(
      children: <Widget>[
        _header(lang.newMnemonicTitle, () => Navigator.of(context).pop()),
        SizedBox(height: maxHeight),
        Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            Container(
              width: 600.0,
              child: Row(children: [
                  Expanded(
                    child: Container(
                      height: 45.0,
                      padding: EdgeInsets.only(left: 20, right: 10),
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
                  )),
                  SizedBox(width: 20.0),
                  Container(
                    width: 100.0,
                    child: GestureDetector(
                      onTap: genMnemonic,
                      child: Container(
                        height: 45.0,
                        decoration: BoxDecoration(
                          color: Color(0xFF6174FF),
                          borderRadius: BorderRadius.circular(10.0)),
                        child: Center(
                          child: Text(lang.newMnemonicInput,
                            style: TextStyle(
                              fontSize: 16.0,
                              color: Colors.white,
                        ))),
                  ))),
            ])),
            const SizedBox(height: 32.0),
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
              )
            ),
            const SizedBox(height: 32.0),
            ButtonText(
              text: lang.next,
              enable: _mnemonicChecked,
              action: () {
                setState(() {
                    this._isAccount = true;
                });
            }),
            _footer(lang.hasAccount, () => Navigator.of(context).pop()),
        ])
    ]);
  }

  Widget _accountState(ColorScheme color, AppLocalizations lang) {
    double maxHeight = (MediaQuery.of(context).size.height - 400) / 2;
    if (maxHeight < 20.0) {
      maxHeight = 20.0;
    }

    return Column(crossAxisAlignment: CrossAxisAlignment.center, children: <
      Widget>[
        _header(lang.newAccountTitle, () => setState(() { this._isAccount = false; })),
        SizedBox(height: maxHeight),
        Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: <Widget>[
            newAccountAvatar(color, lang),
            const SizedBox(height: 32.0),
            Container(
              height: 50.0,
              width: 600.0,
              padding: const EdgeInsets.symmetric(horizontal: 20.0),
              decoration: BoxDecoration(
                color: color.surface,
                border: Border.all(
                  color: _nameFocus.hasFocus ? color.primary : color.surface),
                borderRadius: BorderRadius.circular(10.0),
              ),
              child: TextField(
                style: TextStyle(fontSize: 16.0),
                decoration: InputDecoration(
                  border: InputBorder.none,
                  hintText: lang.newAccountName,
                ),
                controller: _nameController,
                focusNode: _nameFocus,
                onChanged: (value) {
                  if (value.length > 0) {
                    _registerChecked = true;
                  } else {
                    _registerChecked = false;
                  }
                  setState(() {});
              }),
            ),
            const SizedBox(height: 32.0),
            ButtonText(text: this._loading ? lang.waiting : lang.ok,
              action: () => registerNewAction(lang.setPin),
              enable: this._registerChecked && !this._loading),
            _footer(lang.hasAccount, () => Navigator.of(context).pop()),
        ])
    ]);
  }

  Widget newAccountAvatar(color, lang) {
    return Container(
      width: 100,
      height: 100,
      decoration: BoxDecoration(
        color: color.surface,
        image: _imageBytes != null ? DecorationImage(
          image: MemoryImage(_imageBytes!),
          fit: BoxFit.cover,
        ) : null,
        borderRadius: BorderRadius.circular(15.0)),
      child: Stack(
        alignment: Alignment.center,
        children: <Widget>[
          if (_imageBytes == null)
          Icon(Icons.camera_alt, size: 47.0, color: Color(0xFFADB0BB)),
          Positioned(
            bottom: -1.0,
            right: -1.0,
            child: InkWell(
              child: Container(
                decoration: ShapeDecoration(
                  color: color.surface,
                  shape: CircleBorder(),
                ),
                child: Icon(Icons.add_circle,
                  size: 32.0, color: color.primary),
              ),
              onTap: () => selectAvatar(context, (bytes) => setState(() {
                    _imageBytes = bytes;
              })),
            ),
          ),
        ],
      ),
    );
  }
}

Widget _header(String value, VoidCallback callback) {
  return Container(
    width: 700.0,
    child: Row(
      mainAxisAlignment: MainAxisAlignment.start,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        GestureDetector(
          onTap: callback,
          child: Container(
            width: 40.0,
            height: 40.0,
            decoration: BoxDecoration(
              color: Color(0xFF6174FF),
              borderRadius: BorderRadius.circular(10.0)),
            child: Center(child: Icon(Icons.arrow_back, color: Colors.white)),
        )),
        const SizedBox(width: 32.0),
        Text(
          value,
          style: TextStyle(
            fontSize: 20.0,
            fontWeight: FontWeight.bold,
          ),
        ),
  ]));
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
