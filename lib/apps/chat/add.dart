import 'dart:async';

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
import 'package:esse/apps/chat/provider.dart';
import 'package:esse/apps/domain/models.dart';

class ChatAddPage extends StatefulWidget {
  final String id;
  final String addr;
  final String name;

  ChatAddPage({Key? key, this.id = '', this.addr = '', this.name = ''}) : super(key: key);

  @override
  _ChatAddPageState createState() => _ChatAddPageState();
}

class _ChatAddPageState extends State<ChatAddPage> {
  bool _showHome = true;
  Widget _coreScreen = Text('');

  void _scanCallback(bool isOk, String app, List params) {
    Navigator.of(context).pop();
    if (isOk && app == 'add-friend' && params.length == 3) {
      setState(() {
          this._showHome = false;
          this._coreScreen = _InfoScreen(
            callback: this._sendCallback,
            id: params[0],
            addr: params[1],
            name: params[2],
            bio: ''
          );
      });
    }
  }

  void _searchCallBack(String id, String addr, String name, String bio) {
    setState(() {
        this._showHome = false;
        this._coreScreen = _InfoScreen(
          callback: this._sendCallback,
          id: id,
          addr: addr,
          name: name,
          bio: bio
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
          contentPadding: EdgeInsets.symmetric(horizontal: 20.0, vertical: 4.0),
          leading: Icon(Icons.create, color: color.primary),
          title: Text(lang.input),
          trailing: Icon(Icons.keyboard_arrow_right, size: 30.0),
          onTap: () => setState(() {
              this._showHome = false;
              this._coreScreen = _InputScreen(callback: this._sendCallback);
          }),
        ),
        ListTile(
          contentPadding: EdgeInsets.symmetric(horizontal: 20.0, vertical: 4.0),
          leading: Icon(Icons.search, color: color.primary),
          title: Text(lang.domainSearch),
          trailing: Icon(Icons.keyboard_arrow_right, size: 30.0),
          onTap: () => setState(() {
              this._showHome = false;
              this._coreScreen = _DomainSearchScreen(callback: this._searchCallBack);
          }),
        ),
        ListTile(
          contentPadding: EdgeInsets.symmetric(horizontal: 20.0, vertical: 4.0),
          leading: Icon(Icons.camera_alt, color: color.primary),
          title: Text(lang.scanQr),
          trailing: Icon(Icons.keyboard_arrow_right, size: 30.0),
          onTap: () => Navigator.push(
            context,
            MaterialPageRoute(builder: (context) => QRScan(callback: this._scanCallback))
          )
        ),
        ListTile(
          contentPadding: EdgeInsets.symmetric(horizontal: 20.0, vertical: 4.0),
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
    new Future.delayed(Duration.zero, () {
        context.read<ChatProvider>().requestList();
        if (widget.id != '') {
          setState(() {
              this._showHome = false;
              this._coreScreen = _InfoScreen(
                callback: this._sendCallback,
                name: widget.name,
                id: widget.id,
                addr: widget.addr,
                bio: '',
              );
          });
        }
    });
  }

  @override
  Widget build(BuildContext context) {
    final isDesktop = isDisplayDesktop(context);
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    if (this._showHome) {
      this._coreScreen = _coreShow(color, lang);
    }

    final provider = context.watch<ChatProvider>();
    final requests = provider.requests;
    final account = context.read<AccountProvider>().activedAccount;
    final requestKeys = requests.keys.toList().reversed.toList(); // it had sorted.

    return Scaffold(
      appBar: AppBar(
        title: Text(lang.addFriend),
        bottom: PreferredSize(
          child: Container(color: const Color(0x40ADB0BB), height: 1.0),
          preferredSize: Size.fromHeight(1.0)
        ),
        leading: isDesktop
        ? IconButton(
          onPressed: () {
            context.read<ChatProvider>().requestClear();
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
                id: account.id, name: account.name, addr: Global.addr)
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
              if (this._showHome && requests.isNotEmpty)
              ListView.builder(
                itemCount: requestKeys.length,
                shrinkWrap: true,
                physics: ClampingScrollPhysics(),
                scrollDirection: Axis.vertical,
                itemBuilder: (BuildContext context, int index) =>
                _RequestItem(request: requests[requestKeys[index]]!),
              ),
            ]
          )
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
          _selectedProvider = provider.id;
        }
        this._providers.add(provider);
    });
    setState(() {});
  }

  _searchResult(List params) {
    print(params);

    if (params.length == 5) {
      widget.callback(
        "EHAAAA...AAAAAAA",
        '0xaaaaaaa....aaaaaaaaa',
        _nameController.text.trim(),
        'aaa'
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
                        hint: Text(lang.loginChooseAccount, style: TextStyle(fontSize: 16)),
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
                rpc.send('domain-search', [name]);
                this._waiting = true;
                this._searchNone = false;
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
    var id = userIdEditingController.text;
    if (id == '') {
      return;
    }

    if (id.substring(0, 2) == 'EH') {
      id = id.substring(2);
    }

    var addr = addrEditingController.text;
    if (addr.substring(0, 2) == '0x') {
      //substring(2); if has 0x, need remove
      addr = addr.substring(2);
    }
    var name = nameEditingController.text;
    var remark = remarkEditingController.text;

    context.read<ChatProvider>().requestCreate(Request(id, addr, name, remark));
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
          text: lang.id,
          controller: userIdEditingController,
          focus: userIdFocus),
        const SizedBox(height: 20.0),
        InputText(
          icon: Icons.location_on,
          text: lang.address,
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

  const _InfoScreen({
      Key? key,
      required this.callback,
      required this.id,
      required this.addr,
      required this.name,
      required this.bio,
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
          Avatar(name: this.name, width: 100.0, colorSurface: false),
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
              onPressed: () => Clipboard.setData(ClipboardData(text: this.id)),
            )
          ),
          ListTile(
            contentPadding: EdgeInsets.symmetric(horizontal: 20.0, vertical: 2.0),
            leading: Icon(Icons.location_on, color: color.primary),
            title: Text(addrPrint(this.addr), style: TextStyle(fontSize: 16.0)),
            trailing: TextButton(
              child: Icon(Icons.copy, size: 20.0),
              onPressed: () => Clipboard.setData(ClipboardData(text: this.addr)),
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
              context.read<ChatProvider>().requestCreate(
                Request(this.id, this.addr, this.name, '')
              );
              this.callback();
            }
          ),
        ]
      )
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
