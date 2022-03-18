import 'dart:convert' show base64;
import 'dart:typed_data' show Uint8List;
import 'dart:ui' show Locale;

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
import 'package:esse/options.dart';

import 'package:esse/pages/account_domain.dart';
import 'package:esse/apps/device/provider.dart';

class AccountQuickPage extends StatefulWidget {
  const AccountQuickPage({Key? key}) : super(key: key);

  @override
  _AccountQuickPageState createState() => _AccountQuickPageState();
}

class _AccountQuickPageState extends State<AccountQuickPage> {
  bool _registerChecked = false;
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
    final locale = context.read<Options>().locale;

    double maxHeight = (MediaQuery.of(context).size.height - 400) / 2;
    if (maxHeight < 20.0) {
      maxHeight = 20.0;
    }

    return Scaffold(
      body: SafeArea(
        child: Container(
          padding: const EdgeInsets.all(20.0),
          child: SingleChildScrollView(
            child: Column(crossAxisAlignment: CrossAxisAlignment.center, children: <
              Widget>[
                _header(lang.loginQuick, () => Navigator.of(context).pop()),
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
                      action: () => registerNewAction(locale),
                      enable: this._registerChecked && !this._loading),
                    _footer(lang.hasAccount, () => Navigator.of(context).pop()),
                ])
            ])
          )
        )
      ),
    );
  }

  Future<List?> _getMnemonic(Locale locale) async {
    final language = LanguageExtension.fromLocale(locale);
    final res = await httpPost('account-generate', [language.toInt()]);
    if (res.isOk) {
      return [language, res.params[0]];
    } else {
      // TODO tostor error
      print(res.error);
    }
  }

  void registerNewAction(Locale locale) async {
    final lang_mnemonic = await _getMnemonic(locale);
    if (lang_mnemonic == null) {
      return;
    }
    final Language language = lang_mnemonic[0];
    final String mnemonic = lang_mnemonic[1];
    final name = _nameController.text;
    final avatar = _imageBytes != null ? base64.encode(_imageBytes!) : "";
    final lock = '';

    if (!_registerChecked) {
      return;
    }

    setState(() { this._loading = true; });

    // send to core node service by rpc.
    final res = await httpPost('account-create', [
        language.toInt(), mnemonic, "", name, lock, avatar
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
                decoration: const ShapeDecoration(
                  color: Colors.white,
                  shape: CircleBorder(),
                ),
                child: Icon(Icons.add_circle,
                  size: 32.0, color: Color(0xFF6174FF)),
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
