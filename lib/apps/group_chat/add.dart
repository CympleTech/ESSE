import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/input_text.dart';
import 'package:esse/widgets/user_info.dart';
import 'package:esse/widgets/shadow_button.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/qr_scan.dart';
import 'package:esse/global.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/chat/models.dart';
import 'package:esse/apps/chat/provider.dart';
import 'package:esse/apps/group_chat/provider.dart';

class GroupAddPage extends StatefulWidget {
  final String id;
  final String addr;
  final String name;

  GroupAddPage({Key key, this.id = '', this.addr = '', this.name = ''}) : super(key: key);

  @override
  _GroupAddPageState createState() => _GroupAddPageState();
}

class _GroupAddPageState extends State<GroupAddPage> {
  TextEditingController _joinIdController = TextEditingController();
  TextEditingController _joinAddrController = TextEditingController();
  TextEditingController _joinRemarkController = TextEditingController();
  TextEditingController _joinNameController = TextEditingController();
  FocusNode _joinIdFocus = FocusNode();
  FocusNode _joinAddrFocus = FocusNode();
  FocusNode _joinRemarkFocus = FocusNode();

  TextEditingController _createAddrController = TextEditingController();
  TextEditingController _createNameController = TextEditingController();
  TextEditingController _createBioController = TextEditingController();
  TextEditingController _createKeyController = TextEditingController();
  FocusNode _createAddrFocus = FocusNode();
  FocusNode _createNameFocus = FocusNode();
  FocusNode _createBioFocus = FocusNode();
  FocusNode _createKeyFocus = FocusNode();
  int _groupType = 0;
  bool _groupNeedAgree = false;
  bool _groupHasKey = true;
  bool _groupHasNeedAgree = true;
  bool _addrOnline = false;
  bool _addrChecked = false;

  // 0 => encrypted, 1 => common, 2 => open.
  Widget _groupTypeWidget(String text, int value, ColorScheme color) {
    return Row(
      children: [
        Radio(
          value: value,
          groupValue: _groupType,
          onChanged: (n) => setState(() {
              _groupType = n;
              if (n == 0) {
                _groupHasKey = true;
              } else {
                _groupHasKey = false;
              }

              if (n == 2) {
                _groupHasNeedAgree = false;
              } else {
                _groupHasNeedAgree = true;
              }
          }),
        ),
        _groupType == value
        ? Text(text, style: TextStyle(color: color.primary))
        : Text(text),
      ]
    );
  }

  _checkAddrPermission() {
    //
  }

  _checkGroupAddr() {
    String addr = _createAddrController.text;
    if (addr.substring(0, 2) == '0x') {
      addr = addr.substring(2);
    }
    context.read<GroupChatProvider>().check(addr);
  }

  _scanCallback(bool isOk, String app, List params) {
    Navigator.of(context).pop();
    if (isOk && app == 'add-group' && params.length == 3) {
      this._joinIdController.text = params[0];
      this._joinAddrController.text = params[1];
      this._joinNameController.text = params[2];
      setState(() {});
    }
  }

  _join() {
    var id = _joinIdController.text;
    if (id == '' || id == null) {
      return;
    }

    if (id.substring(0, 2) == 'EH') {
      id = id.substring(2);
    }

    var addr = _joinAddrController.text;
    // if has 0x, need remove
    if (addr.substring(0, 2) == '0x') {
      addr = addr.substring(2);
    }
    var name = _joinNameController.text;
    var remark = _joinRemarkController.text;

    context.read<ChatProvider>().requestCreate(Request(id, addr, name, remark));
    setState(() {
        _joinIdController.text = '';
        _joinAddrController.text = '';
        _joinNameController.text = '';
        _joinRemarkController.text = '';
    });
  }

  _create() {
    //
  }

  @override
  void initState() {
    super.initState();
    _joinIdController.text = widget.id;
    _joinAddrController.text = widget.addr;
    _joinNameController.text = widget.name;

    _joinIdFocus.addListener(() {
        setState(() {});
    });
    _joinAddrFocus.addListener(() {
        setState(() {});
    });
    _joinRemarkFocus.addListener(() {
        setState(() {});
    });
    _createAddrFocus.addListener(() {
        setState(() {});
    });
    _createNameFocus.addListener(() {
        setState(() {});
    });
    _createBioFocus.addListener(() {
        setState(() {});
    });
    _createKeyFocus.addListener(() {
        setState(() {});
    });
    new Future.delayed(Duration.zero, () {
        //context.read<ChatProvider>().requestList();
    });
  }

  @override
  Widget build(BuildContext context) {
    final isDesktop = isDisplayDesktop(context);
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final provider = context.watch<ChatProvider>();
    final requests = provider.requests;

    final account = context.read<AccountProvider>().activedAccount;

    final requestKeys = requests.keys.toList().reversed.toList(); // it had sorted.

    return SafeArea(
      child: DefaultTabController(
        initialIndex: 0,
        length: 2,
        child: Scaffold(
          appBar: AppBar(
            title: Row(
              children: [
                if (!isDesktop)
                GestureDetector(
                  onTap: () {
                    context.read<ChatProvider>().requestClear();
                    Navigator.pop(context);
                  },
                  child: Container(
                    width: 20.0,
                    child: Icon(Icons.arrow_back, color: color.primary)),
                ),
                SizedBox(width: 15.0),
                Expanded(
                  child: Text('Add Group Chat',
                    style: TextStyle(fontWeight: FontWeight.bold, fontSize: 20.0)),
                ),
                TextButton(
                  onPressed: () => Navigator.push(
                    context,
                    MaterialPageRoute(builder: (context) => QRScan(callback: _scanCallback))
                  ),
                  child: Text(lang.scanQr, style: TextStyle(fontSize: 16.0)),
                ),
              ],
            ),
            bottom: TabBar(
              tabs: <Widget>[
                Tab(
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.add_box_rounded, color: color.primary),
                      const SizedBox(width: 8.0),
                      Text('Join A Group', style: TextStyle(color: color.primary))
                  ])
                ),
                Tab(
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.create_rounded, color: color.primary),
                      const SizedBox(width: 8.0),
                      Text('Create A Group', style: TextStyle(color: color.primary))
                  ])
                ),
              ],
            ),
          ),
          body: TabBarView(
            children: <Widget>[
              Container(
                padding: const EdgeInsets.all(20),
                child: SingleChildScrollView(
                  child: Form(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.center,
                      children: <Widget>[
                        InputText(
                          icon: Icons.person,
                          text: 'Group ID',
                          controller: _joinIdController,
                          focus: _joinIdFocus),

                        const SizedBox(height: 20.0),
                        InputText(
                          icon: Icons.location_on,
                          text: lang.address,
                          controller: _joinAddrController,
                          focus: _joinAddrFocus),
                        const SizedBox(height: 20.0),
                        InputText(
                          icon: Icons.turned_in,
                          text: lang.remark,
                          controller: _joinRemarkController,
                          focus: _joinRemarkFocus),
                        const SizedBox(height: 20.0),
                        ButtonText(action: _join, text: lang.send, width: 600.0),
                        const SizedBox(height: 20.0),
                        const Divider(height: 1.0, color: Color(0x40ADB0BB)),
                        const SizedBox(height: 10.0),
                        if (requests.isNotEmpty)
                        Container(
                          width: 600.0,
                          child: ListView.builder(
                            itemCount: requestKeys.length,
                            shrinkWrap: true,
                            physics: ClampingScrollPhysics(),
                            scrollDirection: Axis.vertical,
                            itemBuilder: (BuildContext context, int index) =>
                            _RequestItem(request: requests[requestKeys[index]]),
                          ),
                        )
                      ],
                    ),
                  ),
                ),
              ),
              Container(
                padding: const EdgeInsets.all(20),
                child: SingleChildScrollView(
                  child: Form(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.center,
                      children: <Widget>[
                        Container(
                          height: 50.0,
                          width: 600.0,
                          child: Row(
                            children: [
                              Expanded(
                                child: Container(
                                  padding: const EdgeInsets.symmetric(horizontal: 20.0),
                                  decoration: BoxDecoration(
                                    color: color.surface,
                                    border: Border.all(color: _createAddrFocus.hasFocus
                                      ? color.primary : color.surface),
                                    borderRadius: BorderRadius.circular(15.0),
                                  ),
                                  child: TextField(
                                    style: TextStyle(fontSize: 16.0),
                                    decoration: InputDecoration(
                                      border: InputBorder.none,
                                      hintText: lang.address),
                                    controller: _createAddrController,
                                    focusNode: _createAddrFocus,
                                    onSubmitted: (_v) => _checkAddrPermission(),
                                    onChanged: (v) {
                                      if (v.length > 0) {
                                        setState(() {
                                            _addrChecked = true;
                                        });
                                      }
                                  }),
                                ),
                              ),
                              if (_addrOnline)
                              Container(
                                padding: const EdgeInsets.only(left: 8.0),
                                child: Icon(Icons.cloud_done_rounded,
                                  color: Colors.green),
                              ),
                              const SizedBox(width: 8.0),
                              Container(
                                width: 100.0,
                                child: InkWell(
                                  onTap: _addrChecked ? _checkGroupAddr : null,
                                  child: Container(
                                    height: 45.0,
                                    decoration: BoxDecoration(
                                      color: Color(0xFF6174FF),
                                      borderRadius: BorderRadius.circular(15.0)),
                                    child: Center(
                                      child: Text(lang.search,
                                        style: TextStyle(fontSize: 16.0, color: Colors.white))),
                              ))),
                        ])),
                        const SizedBox(height: 8.0),
                        Text('Error Message here', style: TextStyle(fontSize: 14.0, color: Colors.red)),
                        Container(
                          width: 600.0,
                          padding: const EdgeInsets.all(10.0),
                          alignment: Alignment.centerLeft,
                          child: Text('Group Info', textAlign: TextAlign.left, style: TextStyle(fontSize: 20.0, fontWeight: FontWeight.bold)),
                        ),
                        Container(
                          width: 100.0,
                          height: 100.0,
                          margin: const EdgeInsets.symmetric(vertical: 10.0),
                          decoration: BoxDecoration(
                            color: color.surface,
                            borderRadius: BorderRadius.circular(15.0)),
                          child: Stack(
                            alignment: Alignment.center,
                            children: <Widget>[
                              Icon(Icons.camera_alt,
                                size: 47.0, color: Color(0xFFADB0BB)),
                              Positioned(
                                bottom: -1.0,
                                right: -1.0,
                                child: InkWell(
                                  child: Icon(Icons.add_circle,
                                    size: 32.0, color: color.primary),
                                  onTap: null, //() => _getImage(context, account.name, color, lang),
                                )
                              ),
                            ],
                          ),
                        ),
                        Container(
                          padding: EdgeInsets.symmetric(vertical: 10.0),
                          width: 600.0,
                          child: Row(
                            mainAxisSize: MainAxisSize.max,
                            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                            children: [
                              _groupTypeWidget('Encrypted', 0, color),
                              _groupTypeWidget('Common', 1, color),
                              _groupTypeWidget('Open', 2, color),
                            ]
                          )
                        ),
                        Container(
                          padding: EdgeInsets.symmetric(vertical: 10.0),
                          child: InputText(
                            icon: Icons.person,
                            text: 'Group Name',
                            controller: _createNameController,
                            focus: _createNameFocus),
                        ),
                        Container(
                          padding: EdgeInsets.symmetric(vertical: 10.0),
                          child: InputText(
                            icon: Icons.location_on,
                            text: 'Group Bio',
                            controller: _createBioController,
                            focus: _createBioFocus),
                        ),
                        if (_groupHasKey)
                        Container(
                          padding: EdgeInsets.symmetric(vertical: 10.0),
                          child: InputText(
                            icon: Icons.turned_in,
                            text: 'Encrypted Key',
                            controller: _createKeyController,
                            focus: _createKeyFocus),
                        ),
                        if (_groupHasNeedAgree)
                        Container(
                          height: 50.0,
                          width: 600.0,
                          child: Row(
                            children: [
                              Switch(
                                value: _groupNeedAgree,
                                onChanged: (value) {
                                  setState(() {
                                      _groupNeedAgree = value;
                                  });
                                },
                              ),
                              Text('Need Group Manager Agree.')
                            ]
                          ),
                        ),
                        const SizedBox(height: 20.0),
                        ButtonText(action: _create, text: lang.create, width: 600.0),
                        const SizedBox(height: 20.0),
                        const Divider(height: 1.0, color: Color(0x40ADB0BB)),
                        const SizedBox(height: 10.0),
                        if (requests.isNotEmpty)
                        Container(
                          width: 600.0,
                          child: ListView.builder(
                            itemCount: requestKeys.length,
                            shrinkWrap: true,
                            physics: ClampingScrollPhysics(),
                            scrollDirection: Axis.vertical,
                            itemBuilder: (BuildContext context, int index) =>
                            _RequestItem(request: requests[requestKeys[index]]),
                          ),
                        )
                      ],
                    ),
                  ),
                ),
              ),
            ],
      )))
    );
  }
}

class _RequestItem extends StatelessWidget {
  final Request request;

  const _RequestItem({Key key, this.request}) : super(key: key);

  Widget _infoList(icon, color, text) {
    return Container(
      width: 300.0,
      padding: const EdgeInsets.symmetric(vertical: 10.0),
      child: Row(
        children: [
          Icon(icon, size: 20.0, color: color),
          const SizedBox(width: 20.0),
          Expanded(child: Text(text)),
        ]
      ),
    );
  }

  Widget _infoListTooltip(icon, color, text) {
    return Container(
      width: 300.0,
      padding: const EdgeInsets.symmetric(vertical: 10.0),
      child: Row(
        children: [
          Icon(icon, size: 20.0, color: color),
          const SizedBox(width: 20.0),
          Expanded(
            child: Tooltip(
              message: text,
              child: Text(betterPrint(text)),
            )
          )
        ]
      ),
    );
  }

  Widget _info(color, lang, context) {
    return Column(
      mainAxisSize: MainAxisSize.max,
      children: [
        request.showAvatar(100.0),
        const SizedBox(height: 10.0),
        Text(request.name),
        const SizedBox(height: 10.0),
        const Divider(height: 1.0, color: Color(0x40ADB0BB)),
        const SizedBox(height: 10.0),
        _infoListTooltip(Icons.person, color.primary, 'EH' + request.gid.toUpperCase()),
        _infoListTooltip(Icons.location_on, color.primary, "0x" + request.addr),
        _infoList(Icons.turned_in, color.primary, request.remark),
        _infoList(Icons.access_time_rounded, color.primary, request.time.toString()),
        const SizedBox(height: 10.0),
        if (request.over)
        InkWell(
          onTap: () {
            Navigator.pop(context);
            Provider.of<ChatProvider>(context, listen: false).requestDelete(request.id);
          },
          hoverColor: Colors.transparent,
          child: Container(
            width: 300.0,
            padding: const EdgeInsets.symmetric(vertical: 10.0),
            decoration: BoxDecoration(
              border: Border.all(color: color.primary),
              borderRadius: BorderRadius.circular(10.0)),
            child: Center(child: Text(lang.ignore,
                style: TextStyle(fontSize: 14.0))),
          )
        ),
        if (!request.over && !request.isMe)
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
          children: [
            InkWell(
              onTap: () {
                Navigator.pop(context);
                Provider.of<ChatProvider>(context, listen: false).requestReject(request.id);
              },
              hoverColor: Colors.transparent,
              child: Container(
                width: 100.0,
                padding: const EdgeInsets.symmetric(vertical: 10.0),
                decoration: BoxDecoration(
                  border: Border.all(),
                  borderRadius: BorderRadius.circular(10.0)),
                child: Center(child: Text(lang.reject,
                    style: TextStyle(fontSize: 14.0))),
              )
            ),
            InkWell(
              onTap: () {
                Navigator.pop(context);
                Provider.of<ChatProvider>(context, listen: false).requestAgree(request.id);
              },
              hoverColor: Colors.transparent,
              child: Container(
                width: 100.0,
                padding: const EdgeInsets.symmetric(vertical: 10.0),
                decoration: BoxDecoration(
                  border: Border.all(color: color.primary),
                  borderRadius: BorderRadius.circular(10.0)),
                child: Center(child: Text(lang.agree,
                    style: TextStyle(fontSize: 14.0, color: color.primary))),
              )
            ),
          ]
        ),
        if (!request.over && request.isMe)
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
          children: [
            InkWell(
              onTap: () {
                Navigator.pop(context);
                Provider.of<ChatProvider>(context, listen: false).requestDelete(request.id);
              },
              hoverColor: Colors.transparent,
              child: Container(
                width: 100.0,
                padding: const EdgeInsets.symmetric(vertical: 10.0),
                decoration: BoxDecoration(
                  border: Border.all(),
                  borderRadius: BorderRadius.circular(10.0)),
                child: Center(child: Text(lang.ignore,
                    style: TextStyle(fontSize: 14.0))),
              )
            ),
            InkWell(
              onTap: () {
                Navigator.pop(context);
                Provider.of<ChatProvider>(context, listen: false).requestCreate(request);
              },
              hoverColor: Colors.transparent,
              child: Container(
                width: 100.0,
                padding: const EdgeInsets.symmetric(vertical: 10.0),
                decoration: BoxDecoration(
                  border: Border.all(color: color.primary),
                  borderRadius: BorderRadius.circular(10.0)),
                child: Center(child: Text(lang.resend,
                    style: TextStyle(fontSize: 14.0, color: color.primary))),
              )
            ),
          ]
        )
      ]
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () => showShadowDialog(context, Icons.info, lang.info, _info(color, lang, context)),
      child: SizedBox(
        height: 55.0,
        child: Row(
          children: [
            Container(
              width: 45.0,
              height: 45.0,
              margin: const EdgeInsets.only(right: 15.0),
              child: request.showAvatar(),
            ),
            Expanded(
              child: Container(
                height: 55.0,
                child: Row(
                  children: [
                    Expanded(
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(request.name, maxLines: 1, overflow: TextOverflow.ellipsis,
                            style: TextStyle(fontSize: 16.0)),
                          Text(request.remark, maxLines: 1, overflow: TextOverflow.ellipsis,
                            style: TextStyle(color: Color(0xFFADB0BB),
                              fontSize: 12.0)),
                        ],
                      ),
                    ),
                    SizedBox(width: 10.0),
                    if (request.over || request.isMe)
                    Container(
                      child: Text(
                        request.ok ? lang.added : (request.over ? lang.rejected : lang.sended),
                        style: TextStyle(color: Color(0xFFADB0BB), fontSize: 14.0),
                    )),
                    if (!request.over && !request.isMe)
                    InkWell(
                      onTap: () => context.read<ChatProvider>().requestAgree(request.id),
                      hoverColor: Colors.transparent,
                      child: Container(
                        height: 35.0,
                        padding: const EdgeInsets.symmetric(horizontal: 10.0),
                        decoration: BoxDecoration(
                          border: Border.all(color: color.primary),
                          borderRadius: BorderRadius.circular(10.0)),
                        child: Center(child: Text(lang.agree,
                            style: TextStyle(fontSize: 14.0, color: color.primary))),
                      )
                    ),
                  ]
                )
              ),
            ),
          ],
        ),
      ),
    );
  }
}
