import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/widgets/avatar.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/input_text.dart';
import 'package:esse/widgets/user_info.dart';
import 'package:esse/widgets/shadow_button.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/qr_scan.dart';
import 'package:esse/global.dart';
import 'package:esse/provider.dart';
import 'package:esse/rpc.dart';

import 'package:esse/apps/chat/models.dart';
import 'package:esse/apps/chat/list.dart';
import 'package:esse/apps/domain/models.dart';

class ChatAdd extends StatefulWidget {
  final String id;
  final String addr;
  final String name;
  ChatAdd({Key? key, this.id = '', this.addr = '', this.name = ''}) : super(key: key);

  @override
  _ChatAddState createState() => _ChatAddState();
}

class _ChatAddState extends State<ChatAdd> {
  bool _showHome = true;
  Widget _coreScreen = Text('');

  Map<int, Request> _requests = {};

  void _scanCallback(bool isOk, String app, List params) {
    Navigator.of(context).pop();
    if (isOk && app == 'add-friend' && params.length == 3) {
      setState(() {
          this._showHome = false;
          final avatar = Avatar(name: params[2], width: 100.0, colorSurface: false);
          String id = gidParse(params[0].trim());
          String addr = addrParse(params[1]);

          this._coreScreen = _InfoScreen(
            callback: this._sendCallback,
            id: id,
            addr: addr,
            name: params[2],
            bio: '',
            avatar: avatar,
          );
      });
    }
  }

  void _searchCallBack(String id, String addr, String name, String bio, Avatar avatar) {
    setState(() {
        this._showHome = false;
        this._coreScreen = _InfoScreen(
          callback: this._sendCallback,
          id: id,
          addr: addr,
          name: name,
          bio: bio,
          avatar: avatar,
        );
    });
  }

  void _sendCallback() {
    setState(() {
        this._showHome = true;
    });
  }

  void chooseImage() async {
    print('choose qr image');
  }

  Widget _coreShow(ColorScheme color, AppLocalizations lang) {
    return Column(
      children: <Widget>[
        ListTile(
          contentPadding: EdgeInsets.symmetric(horizontal: 10.0, vertical: 4.0),
          leading: Icon(Icons.create, color: color.primary),
          title: Text(lang.input),
          trailing: Icon(Icons.keyboard_arrow_right, size: 30.0),
          onTap: () => setState(() {
              this._showHome = false;
              this._coreScreen = _InputScreen(callback: this._sendCallback);
          }),
        ),
        ListTile(
          contentPadding: EdgeInsets.symmetric(horizontal: 10.0, vertical: 4.0),
          leading: Icon(Icons.search, color: color.primary),
          title: Text(lang.domainSearch),
          trailing: Icon(Icons.keyboard_arrow_right, size: 30.0),
          onTap: () => setState(() {
              this._showHome = false;
              this._coreScreen = _DomainSearchScreen(callback: this._searchCallBack);
          }),
        ),
        ListTile(
          contentPadding: EdgeInsets.symmetric(horizontal: 10.0, vertical: 4.0),
          leading: Icon(Icons.camera_alt, color: color.primary),
          title: Text(lang.scanQr),
          trailing: Icon(Icons.keyboard_arrow_right, size: 30.0),
          onTap: () => Navigator.push(
            context,
            MaterialPageRoute(builder: (context) => QRScan(callback: this._scanCallback))
          )
        ),
        ListTile(
          contentPadding: EdgeInsets.symmetric(horizontal: 10.0, vertical: 4.0),
          leading: Icon(Icons.image, color: color.primary),
          title: Text(lang.scanImage + " (${lang.wip})"),
          trailing: Icon(Icons.keyboard_arrow_right, size: 30.0),
          onTap: () => print('wip'),
        ),
        const SizedBox(
          height: 20.0,
          child: const Divider(height: 1.0, color: Color(0x40ADB0BB)),
        )
      ],
    );
  }

  @override
  void initState() {
    super.initState();

    rpc.addListener('chat-request-create', _requestCreate);
    rpc.addListener('chat-request-delivery', _requestDelivery);
    rpc.addListener('chat-request-agree', _requestAgree);
    rpc.addListener('chat-request-reject', _requestReject);

    new Future.delayed(Duration.zero, () {
        if (widget.id != '') {
          setState(() {
              this._showHome = false;
              final avatar = Avatar(name: widget.name, width: 100.0, colorSurface: false);
              this._coreScreen = _InfoScreen(
                callback: this._sendCallback,
                name: widget.name,
                id: widget.id,
                addr: widget.addr,
                bio: '',
                avatar: avatar,
              );
          });
        }
    });
    _loadRequest();
  }

  _requestCreate(List params) {
    this._requests[params[0]] = Request.fromList(params);
    setState(() {});
  }

  _requestDelivery(List params) {
    final id = params[0];
    final isDelivery = params[1];
    if (this._requests.containsKey(id)) {
      this._requests[id]!.isDelivery = isDelivery;
      setState(() {});
    }
  }

  _requestAgree(List params) {
    final id = params[0]; // request's id.
    if (this._requests.containsKey(id)) {
      this._requests[id]!.overIt(true);
      setState(() {});
    }
  }

  _requestReject(List params) {
    final id = params[0];
    if (this._requests.containsKey(id)) {
      this._requests[id]!.overIt(false);
      setState(() {});
    }
  }

  _loadRequest() async {
    this._requests.clear();
    final res = await httpPost('chat-request-list', []);
    if (res.isOk) {
      res.params.forEach((param) {
          if (param.length == 10) {
            this._requests[param[0]] = Request.fromList(param);
          }
      });
      setState(() {});
    } else {
      print(res.error);
    }
  }

  @override
  Widget build(BuildContext context) {
    final isDesktop = isDisplayDesktop(context);
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    if (this._showHome) {
      this._coreScreen = _coreShow(color, lang);
    }

    final account = context.read<AccountProvider>().activedAccount;
    final requestKeys = this._requests.keys.toList().reversed.toList();

    return Scaffold(
      appBar: AppBar(
        title: Text(lang.addFriend),
        leading: isDesktop
        ? IconButton(
          onPressed: () {
            context.read<AccountProvider>().updateActivedWidget(ChatList());
          },
          icon: Icon(Icons.arrow_back, color: color.primary),
        ) : null,
        actions: [
          TextButton(
            onPressed: () => showShadowDialog(
              context,
              Icons.info,
              lang.info,
              UserInfo(app: 'add-friend',
                id: account.gid, name: account.name, addr: Global.addr)
            ),
            child: Padding(
              padding: const EdgeInsets.only(right: 10.0),
              child: Text(lang.myQrcode, style: TextStyle(fontSize: 16.0))),
          ),
        ]
      ),
      body: Container(
        padding: const EdgeInsets.all(20.0),
        alignment: Alignment.topCenter,
        child: SingleChildScrollView(
          child: Column(
            children: [
              if (!this._showHome)
              Container(
                width: 600.0,
                alignment: Alignment.topRight,
                child: IconButton(
                  icon: Icon(Icons.clear, color: color.primary),
                  onPressed: () => setState(() {
                      this._showHome = true;
                })),
              ),
              this._coreScreen,
              if (this._showHome && this._requests.isNotEmpty)
              ListView.builder(
                itemCount: requestKeys.length,
                shrinkWrap: true,
                physics: ClampingScrollPhysics(),
                scrollDirection: Axis.vertical,
                itemBuilder: (BuildContext context, int index) =>
                _item(this._requests[requestKeys[index]]!, color, lang),
              ),
            ]
          )
        ),
      ),
    );
  }

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

  Widget _info(request, color, lang) {
    return Column(
      mainAxisSize: MainAxisSize.max,
      children: [
        request.showAvatar(100.0),
        const SizedBox(height: 10.0),
        Text(request.name),
        const SizedBox(height: 10.0),
        const Divider(height: 1.0, color: Color(0x40ADB0BB)),
        const SizedBox(height: 10.0),
        _infoListTooltip(Icons.person, color.primary, gidText(request.gid), gidPrint(request.gid)),
        _infoListTooltip(Icons.location_on, color.primary, addrText(request.addr), addrPrint(request.addr)),
        _infoList(Icons.turned_in, color.primary, request.remark),
        _infoList(Icons.access_time_rounded, color.primary, request.time.toString()),
        const SizedBox(height: 10.0),
        if (request.over)
        InkWell(
          onTap: () {
            Navigator.pop(context);
            rpc.send('chat-request-delete', [request.id]);
            setState(() {
                this._requests.remove(request.id);
            });
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
                rpc.send('chat-request-reject', [request.id]);
                setState(() {
                    this._requests[request.id]!.overIt(false);
                });
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
                rpc.send('chat-request-agree', [request.id]);
                setState(() {
                    this._requests[request.id]!.overIt(true);
                });
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
                rpc.send('chat-request-delete', [request.id]);
                setState(() {
                    this._requests.remove(request.id);
                });
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
                rpc.send('chat-request-create', [
                    request.gid, request.addr, request.name, request.remark
                ]);
                setState(() {
                    this._requests.remove(request.id);
                });
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

  Widget _item(request, color, lang) {
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () => showShadowDialog(context, Icons.info, lang.info, _info(request, color, lang)),
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
                      onTap: () {
                        rpc.send('chat-request-agree', [request.id]);
                        setState(() {
                            this._requests[request.id]!.overIt(true);
                        });
                      },
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

class _DomainSearchScreen extends StatefulWidget {
  final Function callback;
  const _DomainSearchScreen({Key? key, required this.callback}) : super(key: key);

  @override
  _DomainSearchScreenState createState() => _DomainSearchScreenState();
}

class _DomainSearchScreenState extends State<_DomainSearchScreen> {
  TextEditingController _nameController = TextEditingController();
  FocusNode _nameFocus = FocusNode();
  int? _selectedProvider = null;
  List<ProviderServer> _providers = [];
  bool _waiting = false;
  bool _searchNone = false;

  _domainList(List params) {
    this._providers.clear();
    params[0].forEach((param) {
        final provider = ProviderServer.fromList(param);
        if (provider.isDefault) {
          this._selectedProvider = provider.id;
        }
        this._providers.add(provider);
    });

    if (this._selectedProvider == null && this._providers.length > 0) {
      this._selectedProvider = this._providers[0].id;
    }

    setState(() {});
  }

  _searchResult(List params) {
    if (params.length == 5) {
      String name = params[0].trim();
      Avatar avatar = Avatar(name: name, width: 100.0, colorSurface: false);
      if (params[4].length > 0) {
        avatar = Avatar(
          name: name,
          avatar: base64.decode(params[4]),
          width: 100.0,
          colorSurface: false
        );
      }

      widget.callback(
        params[1],
        params[2],
        name,
        params[3],
        avatar,
      );
    } else {
      setState(() {
          this._waiting = false;
          this._searchNone = true;
      });
    }
  }

  @override
  void initState() {
    super.initState();

    rpc.addListener('domain-list', _domainList, false);
    rpc.addListener('domain-search', _searchResult, false);
    rpc.send('domain-list', []);
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Column(
      children: [
        const SizedBox(height: 20.0),
        Container(
          width: 600.0,
          height: 50.0,
          margin: const EdgeInsets.only(bottom: 20.0),
          padding: const EdgeInsets.only(left: 10.0),
          child: Row(
            children: [
              Text(lang.domainProvider,
                style: TextStyle(color: color.primary, fontWeight: FontWeight.bold)),
              Expanded(
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 20.0),
                  margin: const EdgeInsets.only(left: 20.0),
                  decoration: BoxDecoration(
                    color: color.surface, borderRadius: BorderRadius.circular(10.0)),
                  child: DropdownButtonHideUnderline(
                    child: Theme(
                      data: Theme.of(context).copyWith(
                        canvasColor: color.surface,
                      ),
                      child: DropdownButton<int>(
                        hint: Text('-', style: TextStyle(fontSize: 16)),
                        iconEnabledColor: Color(0xFFADB0BB),
                        value: _selectedProvider,
                        onChanged: (int? m) {
                          if (m != null) {
                            setState(() {
                                _selectedProvider = m;
                            });
                          }
                        },
                        items: this._providers.map((ProviderServer m) {
                            return DropdownMenuItem<int>(
                              value: m.id,
                              child: Text(m.name, style: TextStyle(fontSize: 16))
                            );
                        }).toList(),
                    )),
                )),
              )
            ]
        )),
        InputText(
          icon: Icons.account_box,
          text: lang.domainName,
          controller: this._nameController,
          focus: this._nameFocus),
        SizedBox(
          height: 40.0,
          child: Center(child: Text(this._searchNone ? lang.notExist : '',
              style: TextStyle(color: Colors.red))),
        ),
        ButtonText(
          enable: !this._waiting,
          action: () => setState(() {
              final name = this._nameController.text.trim();
              if (name.length > 0) {
                String addr = '';
                this._providers.forEach((v) {
                    if (v.id == this._selectedProvider) {
                      addr = v.addr;
                    }
                });
                if (addr.length > 0) {
                  rpc.send('domain-search', [addr, name]);
                  this._waiting = true;
                  this._searchNone = false;
                }
              }
        }), text: this._waiting ? lang.waiting : lang.search, width: 600.0),
      ]
    );
  }
}

class _InputScreen extends StatefulWidget {
  final Function callback;
  const _InputScreen({Key? key, required this.callback}) : super(key: key);

  @override
  _InputScreenState createState() => _InputScreenState();
}

class _InputScreenState extends State<_InputScreen> {
  TextEditingController userIdEditingController = TextEditingController();
  TextEditingController addrEditingController = TextEditingController();
  TextEditingController remarkEditingController = TextEditingController();
  TextEditingController nameEditingController = TextEditingController();
  FocusNode userIdFocus = FocusNode();
  FocusNode addrFocus = FocusNode();
  FocusNode remarkFocus = FocusNode();

  send() {
    final id = gidParse(userIdEditingController.text.trim());
    final addr = addrParse(addrEditingController.text.trim());
    if (id == '' || addr == '') {
      return;
    }

    final name = nameEditingController.text.trim();
    final remark = remarkEditingController.text.trim();

    rpc.send('chat-request-create', [id, addr, name, remark]);
    setState(() {
        userIdEditingController.text = '';
        addrEditingController.text = '';
        nameEditingController.text = '';
        remarkEditingController.text = '';
    });

    // return to the add home.
    widget.callback();
  }

  @override
  Widget build(BuildContext context) {
    final lang = AppLocalizations.of(context);

    return Column(
      children: [
        const SizedBox(height: 20.0),
        InputText(
          icon: Icons.person,
          text: lang.id + ' (EH00..00)',
          controller: userIdEditingController,
          focus: userIdFocus),
        const SizedBox(height: 20.0),
        InputText(
          icon: Icons.location_on,
          text: lang.address + ' (0x00..00)',
          controller: addrEditingController,
          focus: addrFocus),
        const SizedBox(height: 20.0),
        InputText(
          icon: Icons.turned_in,
          text: lang.remark,
          controller: remarkEditingController,
          focus: remarkFocus),
        const SizedBox(height: 20.0),
        ButtonText(action: send, text: lang.send, width: 600.0),
      ]
    );
  }
}

class _InfoScreen extends StatelessWidget {
  final Function callback;
  final String id;
  final String addr;
  final String name;
  final String bio;
  final Avatar avatar;

  const _InfoScreen({
      Key? key,
      required this.callback,
      required this.id,
      required this.addr,
      required this.name,
      required this.bio,
      required this.avatar,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Container(
      margin: const EdgeInsets.only(top: 20.0),
      padding: const EdgeInsets.only(top: 30.0, bottom: 20.0),
      decoration: BoxDecoration(color: color.surface,
        borderRadius: BorderRadius.circular(15.0)),
      width: 600.0,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          avatar,
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 10.0),
            child: Text(this.name, style: TextStyle(fontWeight: FontWeight.bold)),
          ),
          const Divider(height: 20.0, color: Color(0x40ADB0BB)),
          ListTile(
            contentPadding: EdgeInsets.symmetric(horizontal: 20.0, vertical: 2.0),
            leading: Icon(Icons.person, color: color.primary),
            title: Text(gidPrint(this.id), style: TextStyle(fontSize: 16.0)),
            trailing: TextButton(
              child: Icon(Icons.copy, size: 20.0),
              onPressed: () => Clipboard.setData(ClipboardData(text: gidText(this.id))),
            )
          ),
          ListTile(
            contentPadding: EdgeInsets.symmetric(horizontal: 20.0, vertical: 2.0),
            leading: Icon(Icons.location_on, color: color.primary),
            title: Text(addrPrint(this.addr), style: TextStyle(fontSize: 16.0)),
            trailing: TextButton(
              child: Icon(Icons.copy, size: 20.0),
              onPressed: () => Clipboard.setData(ClipboardData(text: addrText(this.addr))),
            )
          ),
          ListTile(
            contentPadding: EdgeInsets.symmetric(horizontal: 20.0, vertical: 2.0),
            leading: Icon(Icons.turned_in, color: color.primary),
            title: Text(this.bio, style: TextStyle(fontSize: 16.0)),
          ),
          const Divider(height: 32.0, color: Color(0x40ADB0BB)),
          TextButton(
            child: Text(lang.addFriend, style: TextStyle(fontSize: 20.0)),
            onPressed: () {
              rpc.send('chat-request-create', [this.id, this.addr, this.name, '']);
              this.callback();
            }
          ),
        ]
      )
    );
  }
}
