import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:qr_flutter/qr_flutter.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/better_print.dart';

class UserInfo extends StatefulWidget {
  final String? app;
  final String id;
  final String name;
  final String? title;
  final String? remark;
  final String? bio;
  final VoidCallback? callback;
  final bool showQr;
  final Widget? avatar;
  Map? qrInfo;
  final String pre;

  UserInfo({Key? key,
      required this.id,
      required this.name,
      this.app,
      this.remark,
      this.bio,
      this.callback,
      this.avatar,
      this.title,
      this.showQr = true,
      this.pre = 'EH',
  }) : super(key: key) {
    if (this.showQr) {
      this.qrInfo = {
        "app": this.app,
        "params": [pidText(this.id, this.pre), this.name],
      };
    }
  }

  @override
  _UserInfoState createState() => _UserInfoState();
}

class _UserInfoState extends State<UserInfo> {
  bool idCopy = false;

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);


    Color idColor = idCopy ? color.primary : color.onPrimary;
    print(pidText(widget.id));

    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        if (widget.avatar != null)
        widget.avatar!,
        const SizedBox(height: 10.0),
        Text(widget.name, style: TextStyle(fontSize: 16.0, fontWeight: FontWeight.bold)),
        const SizedBox(height: 10),
        if (widget.showQr)
        Container(
          width: 200.0,
          margin: const EdgeInsets.only(bottom: 8.0),
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
                data: json.encode(widget.qrInfo!),
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
        if (widget.title != null)
        Text(widget.title!, style: TextStyle(fontSize: 16.0, fontStyle: FontStyle.italic)),
        const SizedBox(height: 10),
        const Divider(height: 1.0, color: Color(0x40ADB0BB)),
        const SizedBox(height: 20),
        InkWell(
          onTap: () {
            Clipboard.setData(ClipboardData(text: pidText(widget.id)));
            setState(() {
                idCopy = true;
            });
          },
          child: Container(
            width: 250.0,
            child: Row(
              children: [
                Expanded(
                  child: Text(pidText(widget.id, widget.pre),
                    style: TextStyle(fontSize: 14, color: idColor))),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 10.0),
                  child: Icon(idCopy ? Icons.file_copy : Icons.copy,
                    size: 20.0, color: color.primary),
                ),
              ]
            ),
          )
        ),
        const SizedBox(height: 16),
        if (widget.remark != null)
        Container(
          width: 250.0,
          padding: const EdgeInsets.only(top: 16.0),
          child: Row(
            children: [
              Icon(Icons.turned_in, size: 20.0, color: color.primary),
              const SizedBox(width: 16.0),
              Expanded(
                child: Center(child: Text(widget.remark!, style: TextStyle(fontSize: 14))),
              )
            ]
          ),
        ),
        if (widget.bio != null)
        Container(
          width: 250.0,
          padding: const EdgeInsets.only(top: 16.0),
          child: Row(
            children: [
              Icon(Icons.campaign, size: 20.0, color: color.primary),
              const SizedBox(width: 16.0),
              Expanded(
                child: Center(child: Text(widget.bio!, style: TextStyle(fontSize: 14))),
              )
            ]
          ),
        ),
        const SizedBox(height: 16),
        if (widget.callback != null)
        Container(
          width: 250.0,
          padding: const EdgeInsets.symmetric(vertical: 10.0),
          child: InkWell(
            onTap: widget.callback!,
            hoverColor: Colors.transparent,
            child: Container(
              padding: const EdgeInsets.symmetric(vertical: 10.0),
              decoration: BoxDecoration(border: Border.all(color: color.primary),
                borderRadius: BorderRadius.circular(10.0)),
              child: Center(child: Text(lang.add, style: TextStyle(fontSize: 14.0, color: color.primary))),
            )
          ),
        ),
      ]
    );
  }
}
