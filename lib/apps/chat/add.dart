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
import 'package:esse/apps/chat/list.dart';
import 'package:esse/apps/chat/provider.dart';

class ChatAddPage extends StatefulWidget {
  final String id;
  final String addr;
  final String name;

  ChatAddPage({Key? key, this.id = '', this.addr = '', this.name = ''}) : super(key: key);

  @override
  _ChatAddPageState createState() => _ChatAddPageState();
}

class _ChatAddPageState extends State<ChatAddPage> {
  TextEditingController userIdEditingController = TextEditingController();
  TextEditingController addrEditingController = TextEditingController();
  TextEditingController remarkEditingController = TextEditingController();
  TextEditingController nameEditingController = TextEditingController();
  FocusNode userIdFocus = FocusNode();
  FocusNode addrFocus = FocusNode();
  FocusNode remarkFocus = FocusNode();

  scanCallback(bool isOk, String app, List params) {
    Navigator.of(context).pop();
    if (isOk && app == 'add-friend' && params.length == 3) {
      this.userIdEditingController.text = params[0];
      this.addrEditingController.text = params[1];
      this.nameEditingController.text = params[2];
      setState(() {});
    }
  }

  Future chooseImage() async {
    print('choose qr image');
  }

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
  }

  @override
  void initState() {
    super.initState();
    userIdEditingController.text = widget.id;
    addrEditingController.text = widget.addr;
    nameEditingController.text = widget.name;

    userIdFocus.addListener(() {
        setState(() {});
    });
    addrFocus.addListener(() {
        setState(() {});
    });
    remarkFocus.addListener(() {
        setState(() {});
    });
    new Future.delayed(Duration.zero, () {
        context.read<ChatProvider>().requestList();
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
            child: Text(lang.myQrcode, style: TextStyle(fontSize: 16.0)),
          ),
        ]
      ),
      body: Container(
        padding: const EdgeInsets.all(10.0),
        alignment: Alignment.topCenter,
        child: SingleChildScrollView(
          child: Container(
            width: 600,
            padding: const EdgeInsets.all(20),
            child: Column(
              children: <Widget>[
                Container(
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                    children: [
                      ShadowButton(
                        icon: Icons.camera_alt,
                        color: color,
                        text: lang.scanQr,
                        action: () => Navigator.push(
                          context,
                          MaterialPageRoute(builder: (context) => QRScan(callback: scanCallback))
                      )),
                      if (MediaQuery.of(context).size.width < 400) Spacer(),
                      ShadowButton(
                        icon: Icons.image,
                        color: color,
                        text: lang.scanImage,
                        action: chooseImage),
                    ],
                  ),
                ),
                const SizedBox(height: 40.0),
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
                const SizedBox(height: 20.0),
                const Divider(height: 1.0, color: Color(0x40ADB0BB)),
                const SizedBox(height: 10.0),
                if (requests.isNotEmpty)
                ListView.builder(
                  itemCount: requestKeys.length,
                  shrinkWrap: true,
                  physics: ClampingScrollPhysics(),
                  scrollDirection: Axis.vertical,
                  itemBuilder: (BuildContext context, int index) =>
                  _RequestItem(request: requests[requestKeys[index]]!),
                ),
              ],
            ),
          ),
        ),
      ),
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
