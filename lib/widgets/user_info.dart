import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:qr_flutter/qr_flutter.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/better_print.dart';

class UserInfo extends StatefulWidget {
  final String id;
  final String name;
  final String addr;
  Map qrInfo;

  UserInfo({Key key, this.id, this.name, this.addr}) : super(key: key) {
    this.qrInfo = {
      "app": "add-friend",
      "params": [this.id, this.addr, this.name],
    };
  }

  @override
  _UserInfoState createState() => _UserInfoState();
}

class _UserInfoState extends State<UserInfo> {
  bool idCopy = false;
  bool addrCopy = false;

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);


    Color idColor = idCopy ? color.primary : color.onPrimary;
    Color addrColor = addrCopy ? color.primary : color.onPrimary;

    return Column(
      children: [
        Container(
          width: 200.0,
          padding: const EdgeInsets.all(2.0),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(5.0),
            border: Border.all(color: Color(0x40ADB0BB)),
            color: Colors.white,
          ),
          child: Stack(
            alignment:Alignment.center,
            children: [
              QrImage(
                data: json.encode(widget.qrInfo),
                version: QrVersions.auto,
                foregroundColor: Colors.black,
              ),
              Container(
                height: 44,
                width: 44,
                padding: EdgeInsets.all(2.0),
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(10.0),
                  border: Border.all(color: Color(0x40ADB0BB)),
                  color: Colors.white,
                ),
                child: Container(
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(10.0),
                    image: DecorationImage(
                      image: AssetImage('assets/logo/logo_40.jpg'),
                    ),
                  ),
                )
              ),
            ]
          )
        ),
        const SizedBox(height: 20),
        Center(child: Text(lang.qrFriend, style: TextStyle(fontSize: 16.0, fontWeight: FontWeight.bold))),
        const SizedBox(height: 20),
        const Divider(height: 1.0, color: Color(0x40ADB0BB)),
        const SizedBox(height: 20),
        InkWell(
          onTap: () {
            Clipboard.setData(ClipboardData(text: widget.id));
            setState(() {
                idCopy = true;
                addrCopy = false;
            });
          },
          child: Container(
            width: 250.0,
            child: Row(
              children: [
                Icon(Icons.person, size: 20.0, color: color.primary),
                Spacer(),
                Text(betterPrint(widget.id), style: TextStyle(fontSize: 14, color: idColor)),
                Spacer(),
                Icon(idCopy ? Icons.file_copy : Icons.copy, size: 20.0, color: color.primary),
              ]
            ),
          )
        ),
        const SizedBox(height: 20),
        InkWell(
          onTap: () {
            Clipboard.setData(ClipboardData(text: widget.addr));
            setState(() {
                idCopy = false;
                addrCopy = true;
            });
          },
          child: Container(
            width: 250.0,
            child: Row(
              children: [
                Icon(Icons.location_on, size: 20.0, color: color.primary),
                Spacer(),
                Text(betterPrint(widget.addr), style: TextStyle(fontSize: 14, color: addrColor)),
                Spacer(),
                Icon(addrCopy ? Icons.file_copy : Icons.copy, size: 20.0, color: color.primary),
              ]
            ),
          )
        ),
      ]
    );
  }
}
