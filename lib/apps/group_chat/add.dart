import 'dart:convert' show base64;
import 'dart:typed_data' show Uint8List;

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
import 'package:esse/widgets/select_avatar.dart';
import 'package:esse/global.dart';
import 'package:esse/rpc.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/group_chat/models.dart';
import 'package:esse/apps/group_chat/list.dart';
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
  TextEditingController _joinNameController = TextEditingController();
  FocusNode _joinIdFocus = FocusNode();
  FocusNode _joinAddrFocus = FocusNode();

  TextEditingController _createAddrController = TextEditingController();
  TextEditingController _createNameController = TextEditingController();
  TextEditingController _createBioController = TextEditingController();
  TextEditingController _createKeyController = TextEditingController();
  FocusNode _createAddrFocus = FocusNode();
  FocusNode _createNameFocus = FocusNode();
  FocusNode _createBioFocus = FocusNode();
  FocusNode _createKeyFocus = FocusNode();
  Uint8List _createAvatarBytes;

  int _groupAddr = 0;
  int _groupType = 1;
  bool _groupNeedAgree = false;
  bool _addrOnline = false;
  bool _addrChecked = false;
  String _myName = '';

  bool _requestsLoadMore = true;

  // 0 => encrypted, 1 => common, 2 => open.
  Widget _groupAddrWidget(String text, int value, ColorScheme color, bool disabled) {
    return Row(
      children: [
        Radio(
          value: value,
          groupValue: _groupAddr,
          onChanged: disabled ? null : (n) => setState(() {
              _groupAddr = n;
          }),
        ),
        _groupAddr == value
        ? Text(text, style: TextStyle(color: color.primary))
        : (disabled ? Text(text, style: TextStyle(color: Color(0xFFADB0BB)))
          : Text(text)),
      ]
    );
  }

  // 0 => encrypted, 1 => common, 2 => open.
  Widget _groupTypeWidget(String text, int value, ColorScheme color, bool disabled) {
    return Row(
      children: [
        Radio(
          value: value,
          groupValue: _groupType,
          onChanged: disabled ? null : (n) => setState(() {
              _groupType = n;
          }),
        ),
        _groupType == value
        ? Text(text, style: TextStyle(color: color.primary))
        : (disabled ? Text(text, style: TextStyle(color: Color(0xFFADB0BB)))
          : Text(text)),
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
    print(app);
    print(params);
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

    if (id.substring(0, 2) == 'EG') {
      id = id.substring(2);
    }

    var addr = _joinAddrController.text;
    // if has 0x, need remove
    if (addr.substring(0, 2) == '0x') {
      addr = addr.substring(2);
    }
    var name = _joinNameController.text;
    context.read<GroupChatProvider>().join(GroupType.Open, id, addr, name, "");
    setState(() {
        _joinIdController.text = '';
        _joinAddrController.text = '';
        _joinNameController.text = '';
    });
  }

  _create() {
    var addr = _createAddrController.text.trim();
    // if has 0x, need remove
    if (addr.substring(0, 2) == '0x') {
      addr = addr.substring(2);
    }
    final name = _createNameController.text.trim();
    final bio = _createBioController.text.trim();
    final avatar = _createAvatarBytes != null ? base64.encode(_createAvatarBytes) : "";
    rpc.send('group-chat-create', [_groupType, _myName, addr, name, bio, _groupNeedAgree, avatar]);
    setState(() {
        _createNameController.text = '';
        _createBioController.text = '';
        _groupNeedAgree = false;
    });
  }

  @override
  void initState() {
    super.initState();
    _addrChecked = false;

    _joinIdController.text = widget.id;
    _joinAddrController.text = widget.addr;
    _joinNameController.text = widget.name;

    _joinIdFocus.addListener(() {
        setState(() {});
    });
    _joinAddrFocus.addListener(() {
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

    rpc.send('group-chat-request-list', [false]);
    new Future.delayed(Duration.zero, () {
        _myName = context.read<AccountProvider>().activedAccount.name;
        context.read<GroupChatProvider>().clearCheck();
        setState(() {});
    });
  }

  @override
  Widget build(BuildContext context) {
    final isDesktop = isDisplayDesktop(context);
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final provider = context.watch<GroupChatProvider>();
    final checks = provider.createCheckType.lang(lang);
    final checkLang = checks[0];
    final checkOk = checks[1];
    provider.createSupported;

    final groups = provider.groups;
    final createKeys = provider.createKeys;

    final requests = provider.requests;
    final requestKeys = requests.keys.toList().reversed.toList();

    return DefaultTabController(
        initialIndex: 0,
        length: 2,
        child: Scaffold(
          appBar: AppBar(
            title: Text(lang.addFriend),
            leading: isDesktop
            ? IconButton(
              onPressed: () {
                context.read<GroupChatProvider>().requestClear();
                context.read<AccountProvider>().updateActivedWidget(GroupChatList());
              },
              icon: Icon(Icons.arrow_back, color: color.primary),
            ) : null,
            actions: [
              TextButton(
                onPressed: () => Navigator.push(context,
                  MaterialPageRoute(builder: (context) => QRScan(callback: _scanCallback))
                ),
                child: Text(lang.scanQr, style: TextStyle(fontSize: 16.0)),
              ),
            ],
            bottom: TabBar(
              tabs: <Widget>[
                Tab(
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.add_box_rounded, color: color.primary),
                      const SizedBox(width: 8.0),
                      Text(lang.groupJoin, style: TextStyle(color: color.primary))
                  ])
                ),
                Tab(
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.create_rounded, color: color.primary),
                      const SizedBox(width: 8.0),
                      Text(lang.groupCreate, style: TextStyle(color: color.primary))
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
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.center,
                    children: <Widget>[
                      const SizedBox(height: 20.0),
                      InputText(
                        icon: Icons.groups,
                        text: lang.groupChatId,
                        controller: _joinIdController,
                        focus: _joinIdFocus),
                      const SizedBox(height: 20.0),
                      InputText(
                        icon: Icons.location_on,
                        text: lang.groupChatAddr,
                        controller: _joinAddrController,
                        focus: _joinAddrFocus),
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
                      ),
                      if (_requestsLoadMore)
                      TextButton(
                        onPressed: () {
                          rpc.send('group-chat-request-list', [true]);
                          setState(() {
                              _requestsLoadMore = false;
                          });
                        },
                        child: Text(lang.loadMore, style: TextStyle(fontSize: 14.0)),
                      ),
                    ],
                  ),
                ),
              ),
              Container(
                padding: const EdgeInsets.all(20),
                child: SingleChildScrollView(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.center,
                    children: <Widget>[
                      Container(
                        width: 600.0,
                        padding: const EdgeInsets.all(10.0),
                        alignment: Alignment.centerLeft,
                        child: Text('1. ' + lang.groupChatAddr, textAlign: TextAlign.left,
                          style: Theme.of(context).textTheme.title),
                      ),
                      Container(
                        padding: EdgeInsets.only(bottom: 10.0),
                        width: 600.0,
                        child: Row(
                          mainAxisSize: MainAxisSize.max,
                          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                          children: [
                            _groupAddrWidget(lang.deviceRemote, 0, color, false),
                            _groupAddrWidget(lang.deviceLocal, 1, color, true),
                          ]
                        )
                      ),
                      if (_groupAddr == 0)
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
                            if (checkOk)
                            Container(
                              padding: const EdgeInsets.only(left: 8.0),
                              child: Icon(Icons.cloud_done_rounded,
                                color: Colors.green),
                            ),
                            const SizedBox(width: 8.0),
                            InkWell(
                              onTap: _addrChecked ? _checkGroupAddr : null,
                              child: Container(
                                padding: const EdgeInsets.symmetric(horizontal: 20.0),
                                height: 45.0,
                                decoration: BoxDecoration(
                                  color: Color(0xFF6174FF),
                                  borderRadius: BorderRadius.circular(15.0)),
                                child: Center(
                                  child: Text(lang.search,
                                    style: TextStyle(fontSize: 16.0, color: Colors.white))),
                            )),
                            const SizedBox(width: 8.0),
                            InkWell(
                              onTap: _addrChecked ? _checkGroupAddr : null,
                              child: Container(
                                padding: const EdgeInsets.symmetric(horizontal: 20.0),
                                height: 45.0,
                                decoration: BoxDecoration(
                                  color: Color(0xFF6174FF),
                                  borderRadius: BorderRadius.circular(15.0)),
                                child: Center(
                                  child: Text(lang.add,
                                    style: TextStyle(fontSize: 16.0, color: Colors.white))),
                            )),
                      ])),
                      const SizedBox(height: 8.0),
                      Text(checkLang, style: TextStyle(fontSize: 14.0,
                          color: checkOk ? Colors.green : Colors.red)),
                      Container(
                        width: 600.0,
                        padding: const EdgeInsets.all(10.0),
                        alignment: Alignment.centerLeft,
                        child: Text('2. ' + lang.groupChatInfo, textAlign: TextAlign.left,
                          style: Theme.of(context).textTheme.title),
                      ),
                      Container(
                        width: 100.0,
                        height: 100.0,
                        margin: const EdgeInsets.symmetric(vertical: 10.0),
                        decoration: BoxDecoration(
                          color: color.surface,
                          image: _createAvatarBytes != null ? DecorationImage(
                            image: MemoryImage(_createAvatarBytes),
                            fit: BoxFit.cover,
                          ) : null,
                          borderRadius: BorderRadius.circular(15.0)),
                        child: Stack(
                          alignment: Alignment.center,
                          children: <Widget>[
                            if (_createAvatarBytes == null)
                            Icon(Icons.camera_alt, size: 48.0, color: Color(0xFFADB0BB)),
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
                                onTap: () => selectAvatar(context, (bytes) => setState(() {
                                      _createAvatarBytes = bytes;
                                })),
                              ),
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
                            _groupTypeWidget(lang.groupTypeEncrypted, 0, color, true),
                            _groupTypeWidget(lang.groupTypePrivate, 1, color, false),
                            _groupTypeWidget(lang.groupTypeOpen, 2, color, false),
                          ]
                        )
                      ),
                      Container(
                        width: 600.0,
                        padding: const EdgeInsets.all(12.0),
                        margin: const EdgeInsets.only(bottom: 10.0),
                        decoration: BoxDecoration(color: color.surface,
                          borderRadius: BorderRadius.circular(15.0)
                        ),
                        child: Text(
                          _groupType == 0 ? lang.groupTypeEncryptedInfo
                          : (_groupType == 1 ? lang.groupTypePrivateInfo
                            : lang.groupTypeOpenInfo),
                          style: TextStyle(fontSize: 14.0, height: 1.5,
                            fontStyle: FontStyle.italic),
                          textAlign: TextAlign.center,
                        ),
                      ),
                      Container(
                        padding: EdgeInsets.symmetric(vertical: 10.0),
                        child: InputText(
                          icon: Icons.account_box,
                          text: lang.groupChatName,
                          controller: _createNameController,
                          focus: _createNameFocus),
                      ),
                      Container(
                        padding: EdgeInsets.symmetric(vertical: 10.0),
                        child: InputText(
                          icon: Icons.campaign,
                          text: lang.groupChatBio,
                          controller: _createBioController,
                          focus: _createBioFocus),
                      ),
                      if (_groupType == 0)
                      Container(
                        padding: EdgeInsets.symmetric(vertical: 10.0),
                        child: InputText(
                          icon: Icons.enhanced_encryption,
                          text: lang.groupChatKey,
                          controller: _createKeyController,
                          focus: _createKeyFocus),
                      ),
                      if (_groupType != 2)
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
                            Text(lang.groupRequireConsent)
                          ]
                        ),
                      ),
                      const SizedBox(height: 20.0),
                      ButtonText(action: _create, text: lang.create, width: 600.0),
                      const SizedBox(height: 20.0),
                      const Divider(height: 1.0, color: Color(0x40ADB0BB)),
                      const SizedBox(height: 10.0),
                      Container(
                        width: 600.0,
                        child: ListView.builder(
                          itemCount: createKeys.length,
                          shrinkWrap: true,
                          physics: ClampingScrollPhysics(),
                          scrollDirection: Axis.vertical,
                          itemBuilder: (BuildContext context, int index) =>
                          _CreateItem(group: groups[createKeys[index]], name: _myName),
                        ),
                      )
                    ],
                  ),
                ),
              ),
            ],
      ))
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
        _infoListTooltip(Icons.person, color.primary,
          (request.isMe ? 'EG' : 'EH') + request.gid.toUpperCase()),
        _infoListTooltip(Icons.location_on, color.primary, "0x" + request.addr),
        _infoList(Icons.turned_in, color.primary, request.remark),
        _infoList(Icons.access_time_rounded, color.primary, request.time.toString()),
        const SizedBox(height: 10.0),
        if (request.over)
        InkWell(
          onTap: () {
            Navigator.pop(context);
            //Provider.of<ChatProvider>(context, listen: false).requestDelete(request.id);
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
                //Provider.of<ChatProvider>(context, listen: false).requestReject(request.id);
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
                //Provider.of<ChatProvider>(context, listen: false).requestAgree(request.id);
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
                //Provider.of<ChatProvider>(context, listen: false).requestDelete(request.id);
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
                //Provider.of<ChatProvider>(context, listen: false).requestCreate(request);
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
                      onTap: () => null, //context.read<ChatProvider>().requestAgree(request.id),
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

class _CreateItem extends StatelessWidget {
  final GroupChat group;
  final String name;
  const _CreateItem({Key key, this.group, this.name}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return SizedBox(
      height: 55.0,
      child: Row(
        children: [
          Container(
            width: 45.0,
            height: 45.0,
            margin: const EdgeInsets.only(right: 15.0),
            child: group.showAvatar(),
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
                        Text(group.name, maxLines: 1, overflow: TextOverflow.ellipsis,
                          style: TextStyle(fontSize: 16.0)),
                        Text(group.bio, maxLines: 1, overflow: TextOverflow.ellipsis,
                          style: TextStyle(color: Color(0xFFADB0BB),
                            fontSize: 12.0)),
                      ],
                    ),
                  ),
                  SizedBox(width: 10.0),
                  group.isOk
                  ? Container(
                    child: Text(
                      lang.added,
                      style: TextStyle(color: Color(0xFFADB0BB), fontSize: 14.0),
                  ))
                  : InkWell(
                    onTap: () => rpc.send('group-chat-resend', [group.id, name]),
                    hoverColor: Colors.transparent,
                    child: Container(
                      height: 35.0,
                      padding: const EdgeInsets.symmetric(horizontal: 10.0),
                      decoration: BoxDecoration(
                        border: Border.all(color: color.primary),
                        borderRadius: BorderRadius.circular(10.0)),
                      child: Center(child: Text(lang.send,
                          style: TextStyle(fontSize: 14.0, color: color.primary))),
                    )
                  ),
                ]
              )
            ),
          ),
        ],
      ),
    );
  }
}
