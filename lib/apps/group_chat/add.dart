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
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/qr_scan.dart';
import 'package:esse/widgets/select_avatar.dart';
import 'package:esse/rpc.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/group_chat/models.dart';
import 'package:esse/apps/group_chat/list.dart';
import 'package:esse/apps/group_chat/provider.dart';

class GroupAddPage extends StatefulWidget {
  final String id;
  final String addr;
  final String name;

  GroupAddPage({Key? key, this.id = '', this.addr = '', this.name = ''}) : super(key: key);

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
  Uint8List? _createAvatarBytes;

  int _groupLocation = 0;
  bool _groupAddLocation = false;
  int _groupType = 1;
  bool _groupNeedAgree = false;
  //bool _addrOnline = false;
  bool _addrChecked = false;
  String _myName = '';

  bool _requestsLoadMore = true;

  Map<int, ProviderServer> _providers = {};
  int _providerSelected = -1;

  _providerList(List params) {
    this._providers.clear();
    int index = 0;
    params.forEach((param) {
        this._providers[index] = ProviderServer.fromList(param);
        index += 1;
    });
    setState(() {});
  }

  _providerCheck(List params) {
    final provider = ProviderServer.fromList(params);
    bool contains = false;
    this._providers.forEach((k, p) {
        if (p.id == provider.id) {
          this._providers[k] = provider;
          contains = true;
        }
    });
    if (!contains) {
      this._providers[this._providers.length] = provider;
    }

    setState(() {});
  }

  _providerDelete(List params) {
    final id = params[0];
    int index = -1;
    this._providers.forEach((k, p) {
        if (p.id == id) {
          index = k;
        }
    });
    if (index > -1) {
      this._providers.remove(index);
    }

    setState(() {});
  }

  // 0 => remote, 1 => local.
  Widget _groupLocationWidget(String text, int value, ColorScheme color, bool disabled) {
    return Row(
      children: [
        Radio(
          value: value,
          groupValue: _groupLocation,
          onChanged: disabled ? null : (int? n) => setState(() {
              _groupLocation = n!;
          }),
        ),
        _groupLocation == value
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
          onChanged: disabled ? null : (int? n) => setState(() {
              _groupType = n!;
          }),
        ),
        _groupType == value
        ? Text(text, style: TextStyle(color: color.primary))
        : (disabled ? Text(text, style: TextStyle(color: Color(0xFFADB0BB)))
          : Text(text)),
      ]
    );
  }

  Widget _providerItem(ProviderServer provider, color, lang, context) {
    return ListTile(
      leading: IconButton(icon: Icon(Icons.sync, color: color.primary),
        onPressed: () => rpc.send('group-chat-provider-check', [
            provider.id, provider.addr
        ])
      ),
      title: Text('group.esse'),
      subtitle: Text('remain 10 times'),
      trailing: IconButton(icon: Icon(Icons.delete, color: Colors.red),
        onPressed: () => showDialog(
          context: context,
          builder: (BuildContext context) {
            return AlertDialog(
              title: Text(lang.delete + " ${provider.name} ?"),
              actions: [
                TextButton(
                  child: Text(lang.cancel),
                  onPressed: () => Navigator.pop(context),
                ),
                TextButton(
                  child: Text(lang.ok),
                  onPressed:  () {
                    Navigator.pop(context);
                    rpc.send('group-chat-provider-delete', [provider.id]);
                    rpc.send('group-chat-provider-list', []);
                  },
                ),
              ]
            );
          },
        )
      ),
    );
  }

  _scanCallback(bool isOk, String app, List params) {
    Navigator.of(context).pop();
    print(app);
    print(params);
    if (isOk && app == 'add-group' && params.length == 3) {
      this._joinIdController.text = gidText(params[0]);
      this._joinAddrController.text = addrText(params[1]);
      this._joinNameController.text = params[2];
      setState(() {});
    }
  }

  _join() {
    final id = gidParse(_joinIdController.text.trim(), 'EG');
    if (id.length < 2) {
      return;
    }
    final addr = addrParse(_joinAddrController.text.trim());
    final name = _joinNameController.text.trim();
    context.read<GroupChatProvider>().join(GroupType.Open, id, addr, name, "");
    setState(() {
        _joinIdController.text = '';
        _joinAddrController.text = '';
        _joinNameController.text = '';
    });
  }

  _create() {
    if (!this._providers.containsKey(this._providerSelected)) {
      return;
    }

    final addr = this._providers[this._providerSelected]!.addr;
    if (_groupLocation == 0 && addr.length < 2) {
      return;
    }
    final name = _createNameController.text.trim();
    final bio = _createBioController.text.trim();
    final avatar = _createAvatarBytes != null ? base64.encode(_createAvatarBytes!) : "";
    rpc.send('group-chat-create', [_groupLocation, _groupType, _myName, addr, name, bio, _groupNeedAgree, avatar]);
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

    _joinIdController.text = gidText(widget.id, 'EG');
    _joinAddrController.text = addrText(widget.addr);
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

    rpc.addListener('group-chat-provider-list', _providerList, false);
    rpc.addListener('group-chat-provider-check', _providerCheck, false);
    rpc.addListener('group-chat-provider-delete', _providerDelete, false);

    rpc.send('group-chat-provider-list', []);
    rpc.send('group-chat-request-list', [false]);

    new Future.delayed(Duration.zero, () {
        _myName = context.read<AccountProvider>().activedAccount.name;
        setState(() {});
    });
  }

  @override
  Widget build(BuildContext context) {
    final isDesktop = isDisplayDesktop(context);
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final provider = context.watch<GroupChatProvider>();

    final groups = provider.groups;
    final createKeys = provider.createKeys;

    final requests = provider.requests;
    final requestKeys = requests.keys.toList().reversed.toList();

    if (this._providerSelected < 0 && this._providers.length > 0) {
      this._providerSelected = 0;
    }
    final maxIndex = this._providers.length - 1;

    return DefaultTabController(
        initialIndex: 0,
        length: 2,
        child: Scaffold(
          appBar: AppBar(
            title: Text(lang.groupChatAdd),
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
                child: Padding(
                  padding: const EdgeInsets.only(right: 10.0),
                  child: Text(lang.scanQr, style: TextStyle(fontSize: 16.0))),
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
                padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 10.0),
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
                      ButtonText(action: _join, text: lang.send),
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
                          _RequestItem(request: requests[requestKeys[index]]!),
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
                padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 10.0),
                child: SingleChildScrollView(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.center,
                    children: <Widget>[
                      Container(
                        width: 600.0,
                        padding: const EdgeInsets.all(10.0),
                        alignment: Alignment.centerLeft,
                        child: Text('1. ' + lang.groupChatLocation, textAlign: TextAlign.left,
                          style: Theme.of(context).textTheme.headline6),
                      ),
                      Container(
                        padding: EdgeInsets.only(bottom: 10.0),
                        width: 600.0,
                        child: Row(
                          mainAxisSize: MainAxisSize.max,
                          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                          children: [
                            _groupLocationWidget(lang.deviceRemote, 0, color, false),
                            _groupLocationWidget(lang.deviceLocal, 1, color, false),
                          ]
                        )
                      ),
                      if (_groupLocation == 0)
                      Container(
                        margin: const EdgeInsets.only(top: 10.0),
                        width: 600.0,
                        child: Row(
                          children: [
                            Padding(
                              padding: const EdgeInsets.symmetric(horizontal: 10.0),
                              child: Text(lang.domainProvider,
                                style: TextStyle(fontWeight: FontWeight.bold)),
                            ),
                            TextButton(child: Icon(Icons.navigate_before),
                              onPressed: this._providerSelected > 0 ? () => setState(() {
                                  this._providerSelected = this._providerSelected - 1;
                              }) : null,
                            ),
                            Expanded(
                              child: Center(
                                child: Text(
                                  this._providerSelected >= 0
                                  ? this._providers[this._providerSelected]!.name
                                  : '',
                                  style: TextStyle(fontWeight: FontWeight.bold, fontSize: 20.0)
                                ),
                            )),
                            TextButton(child: Icon(Icons.navigate_next),
                              onPressed: this._providerSelected < maxIndex ? () => setState(() {
                                  this._providerSelected = this._providerSelected + 1;
                              }) : null,
                            ),
                            const SizedBox(width: 20.0),
                            InkWell(
                              onTap: () => setState(() {
                                  this._groupAddLocation = !this._groupAddLocation;
                              }),
                              child: Container(
                                height: 40.0,
                                padding: const EdgeInsets.symmetric(horizontal: 20.0),
                                decoration: BoxDecoration(
                                  border: Border.all(color: color.primary),
                                  borderRadius: BorderRadius.circular(10.0)),
                                child: Center(
                                  child: Text(this._groupAddLocation ? lang.cancel : lang.add,
                                    style: TextStyle(fontSize: 16.0, color: color.primary))),
                            )),
                      ])),
                      if (this._groupAddLocation)
                      Container(
                        margin: const EdgeInsets.only(top: 10.0),
                        padding: const EdgeInsets.all(20.0),
                        decoration: BoxDecoration(
                          color: color.secondary,
                          borderRadius: BorderRadius.circular(10.0),
                        ),
                        width: 600.0,
                        child: Column(
                          children: [
                            ListTile(
                              leading: Icon(Icons.location_on),
                              title: Container(
                                padding: const EdgeInsets.symmetric(horizontal: 20.0),
                                decoration: BoxDecoration(
                                  color: color.surface,
                                  border: Border.all(color: _createAddrFocus.hasFocus
                                    ? color.primary : color.surface),
                                  borderRadius: BorderRadius.circular(10.0),
                                ),
                                child: TextField(
                                  style: TextStyle(fontSize: 16.0),
                                  decoration: InputDecoration(
                                    border: InputBorder.none,
                                    hintText: lang.address),
                                  controller: _createAddrController,
                                  focusNode: _createAddrFocus,
                                ),
                              ),
                              trailing: IconButton(icon: Icon(Icons.send, color: color.primary),
                                onPressed: () {
                                  final addr = addrParse(_createAddrController.text.trim());
                                  if (addr.length > 0) {
                                    rpc.send('group-chat-provider-check', [0, addr]);
                                  }
                                },
                            )),
                            const Divider(height: 20.0, color: Color(0x40ADB0BB)),
                            Column(
                              children: this._providers.values.map(
                                (provider) => _providerItem(provider, color, lang, context)
                              ).toList(),
                            ),
                          ]
                      )),
                      Container(
                        width: 600.0,
                        padding: const EdgeInsets.all(10.0),
                        alignment: Alignment.centerLeft,
                        child: Text('2. ' + lang.groupChatInfo, textAlign: TextAlign.left,
                          style: Theme.of(context).textTheme.headline6),
                      ),
                      Container(
                        width: 100.0,
                        height: 100.0,
                        margin: const EdgeInsets.symmetric(vertical: 10.0),
                        decoration: BoxDecoration(
                          color: color.surface,
                          image: _createAvatarBytes != null ? DecorationImage(
                            image: MemoryImage(_createAvatarBytes!),
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
                          borderRadius: BorderRadius.circular(10.0)
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
                      ButtonText(action: _create, text: lang.create),
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
                          _CreateItem(group: groups[createKeys[index]]!, name: _myName),
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

  const _RequestItem({Key? key, required this.request}) : super(key: key);

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

  Widget _infoListTooltip(icon, color, text, short) {
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
              child: Text(short),
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
          request.isMe ? gidText(request.gid, 'EG') : gidText(request.gid),
          request.isMe ? gidPrint(request.gid, 'EG') : gidPrint(request.gid),
        ),
        _infoListTooltip(Icons.location_on, color.primary, addrText(request.addr), addrPrint(request.addr)),
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
                          Text(
                            gidPrint(request.gid, 'EG') + " (${addrPrint(request.addr)})",
                            maxLines: 1, overflow: TextOverflow.ellipsis,
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
  const _CreateItem({Key? key, required this.group, required this.name}) : super(key: key);

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
