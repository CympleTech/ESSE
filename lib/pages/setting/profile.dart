import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/show_pin.dart';
import 'package:esse/widgets/select_avatar.dart';
import 'package:esse/global.dart';
import 'package:esse/rpc.dart';
import 'package:esse/provider.dart';

class ProfileDetail extends StatefulWidget {
  ProfileDetail({Key key}) : super(key: key);

  @override
  _ProfileDetailState createState() => _ProfileDetailState();
}

class _ProfileDetailState extends State<ProfileDetail> {
  TextEditingController _nameController = TextEditingController();
  bool _changeName = false;
  bool _mnemoicShow = false;
  List<String> _mnemoicWords = [];

  Widget _infoListTooltip(icon, color, text) {
    return SizedBox(
      width: 300.0,
      height: 40.0,
      child: Row(children: [
        Icon(icon, size: 20.0, color: color),
        const SizedBox(width: 20.0),
        Expanded(
            child: Tooltip(
          message: text,
          child: Text(betterPrint(text)),
        ))
      ]),
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final account = context.watch<AccountProvider>().activedAccount;
    final noImage = account.avatar == null;

    return Wrap(spacing: 20.0, alignment: WrapAlignment.center, children: <
        Widget>[
      Container(
          width: 180.0,
          child: Column(children: [
            Container(
              width: 100.0,
              height: 100.0,
              decoration: noImage
                  ? BoxDecoration(
                      color: color.surface,
                      borderRadius: BorderRadius.circular(15.0))
                  : BoxDecoration(
                      color: color.surface,
                      image: DecorationImage(
                        image: MemoryImage(account.avatar),
                        fit: BoxFit.cover,
                      ),
                      borderRadius: BorderRadius.circular(15.0)),
              child: Stack(
                alignment: Alignment.center,
                children: <Widget>[
                  if (noImage)
                    Icon(Icons.camera_alt,
                        size: 47.0, color: Color(0xFFADB0BB)),
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
                          size: 32.0, color: color.primary),
                      ),
                      onTap: () => selectAvatar(context, (bytes) =>
                        context.read<AccountProvider>().accountUpdate(account.name, bytes),
                      ),
                    ),
                  ),
                ],
              ),
            ),
            _changeName
                ? Padding(
                    padding: const EdgeInsets.symmetric(vertical: 10.0),
                    child: Row(mainAxisSize: MainAxisSize.max, children: [
                      Container(
                        width: 100.0,
                        child: TextField(
                          autofocus: true,
                          style: TextStyle(fontSize: 16.0),
                          textAlign: TextAlign.center,
                          controller: _nameController,
                          decoration: InputDecoration(
                            hintText: account.name,
                            hintStyle: TextStyle(
                                color: Color(0xFF1C1939).withOpacity(0.25)),
                            filled: false,
                            isDense: true,
                          ),
                        ),
                      ),
                      const SizedBox(width: 10.0),
                      GestureDetector(
                        onTap: () {
                          if (_nameController.text.length > 0) {
                            context
                                .read<AccountProvider>()
                                .accountUpdate(_nameController.text);
                          }
                          setState(() {
                            _changeName = false;
                          });
                        },
                        child: Container(
                            width: 20.0,
                            child: Icon(
                              Icons.done_rounded,
                              color: color.primary,
                            )),
                      ),
                      const SizedBox(width: 10.0),
                      GestureDetector(
                        onTap: () => setState(() {
                          _changeName = false;
                        }),
                        child: Container(
                            width: 20.0, child: Icon(Icons.clear_rounded)),
                      ),
                    ]),
                  )
                : Padding(
                    padding: const EdgeInsets.symmetric(vertical: 10.0),
                    child: TextButton(
                      onPressed: () => setState(() {
                        _changeName = true;
                      }),
                      child:
                          Text(account.name, style: TextStyle(fontSize: 16.0)),
                    ),
                  ),
          ])),
      Container(
          width: 300.0,
          child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.max,
              children: [
                _infoListTooltip(Icons.person, color.primary, account.id),
                _infoListTooltip(Icons.location_on, color.primary, Global.addr),
                SizedBox(
                  width: 300.0,
                  height: 40.0,
                  child: Row(children: [
                    Icon(Icons.security_rounded,
                        size: 20.0, color: color.primary),
                    const SizedBox(width: 20.0),
                    TextButton(
                      onPressed: () => _pinCheck(account.lock,
                        () => _changePin(context, account.id, account.lock, lang.setPin),
                        lang.verifyPin,
                      ),
                      child: Text(lang.change + ' PIN'),
                    ),
                  ]),
                ),
                SizedBox(
                  width: 300.0,
                  height: 40.0,
                  child: Row(children: [
                    Icon(Icons.psychology_rounded,
                        size: 20.0, color: color.primary),
                    const SizedBox(width: 20.0),
                    _mnemoicShow
                        ? TextButton(
                            onPressed: () => setState(() {
                              _mnemoicShow = false;
                              _mnemoicWords.clear();
                            }),
                            child: Text(lang.hide + ' ' + lang.mnemonic),
                          )
                        : TextButton(
                            onPressed: () => _pinCheck(account.lock,
                                () => _showMnemonic(account.id, account.lock), lang.verifyPin),
                            child: Text(lang.show + ' ' + lang.mnemonic),
                          ),
                  ]),
                ),
                if (_mnemoicShow)
                  Container(
                    width: 300.0,
                    padding: const EdgeInsets.all(10.0),
                    decoration: BoxDecoration(
                      border: Border.all(color: Color(0x40ADB0BB)),
                      borderRadius: BorderRadius.circular(15),
                    ),
                    child: Wrap(
                      spacing: 10.0,
                      runSpacing: 5.0,
                      alignment: WrapAlignment.center,
                      children: _showMnemonicWords(color),
                    ),
                  )
              ])),
    ]);
  }

  _showMnemonicWords(color) {
    List<Widget> mnemonicWordWidgets = [];
    if (_mnemoicWords.length > 0) {
      _mnemoicWords.asMap().forEach((index, value) {
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
    return mnemonicWordWidgets;
  }

  _changePin(context, String id, String lock, String title) async {
    showShadowDialog(
      context,
      Icons.security_rounded,
      title,
      SetPinWords(callback: (key, lock2) async {
          Navigator.of(context).pop();
          final res = await httpPost(Global.httpRpc, 'account-pin', [lock, lock2]);
          if (res.isOk) {
            Provider.of<AccountProvider>(context, listen: false).accountPin(res.params[0]);
          } else {
            // TODO tostor error
            print(res.error);
          }
      }),
      20.0, // height.
    );
  }

  _showMnemonic(String id, String lock) async {
    final res = await httpPost(Global.httpRpc, 'account-mnemonic', [lock]);
    if (res.isOk) {
      final words = res.params[0];
      _mnemoicWords = words.split(' ');
      setState(() {
        _mnemoicShow = true;
      });
    } else {
      // TODO tostor error
      print(res.error);
    }
  }

  _pinCheck(String hash, Function callback, String title) {
    showShadowDialog(
        context,
        Icons.security_rounded,
        title,
        PinWords(
            hashPin: hash,
            callback: (_key, _hash) async {
              Navigator.of(context).pop();
              callback();
            }));
  }
}
