import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/user_info.dart';
import 'package:esse/widgets/chat_message.dart';
import 'package:esse/provider.dart';
import 'package:esse/session.dart' show SessionType, Session, OnlineType;
import 'package:esse/rpc.dart';

import 'package:esse/apps/primitives.dart';
import 'package:esse/apps/chat_input.dart';
import 'package:esse/apps/chat/models.dart';
import 'package:esse/apps/group/add.dart';

class ChatDetail extends StatefulWidget {
  final int id;
  ChatDetail({Key? key, required this.id}) : super(key: key);

  @override
  _ChatDetailState createState() => _ChatDetailState();
}

class _ChatDetailState extends State<ChatDetail> {
  TextEditingController textController = TextEditingController();
  FocusNode textFocus = FocusNode();
  bool _emojiShow = false;
  bool _sendShow = false;
  bool _menuShow = false;
  bool _recordShow = false;
  String _recordName = '';

  bool _loading = false;
  Friend _friend = Friend('', '', '');
  Map<int, Message> _messages = {};

  @override
  initState() {
    super.initState();

    rpc.addListener('chat-message-list', _messageList, false);
    rpc.addListener('chat-message-create', _messageCreate, true);
    rpc.addListener('chat-message-delivery', _messageDelivery, false);

    textFocus.addListener(() {
        if (textFocus.hasFocus) {
          setState(() {
              _emojiShow = false;
              _menuShow = false;
              _recordShow = false;
          });
        }
    });
  }

  _messageCreate(List params) {
    final msg = Message.fromList(params);
    if (msg.fid == _friend.id) {
      if (!msg.isDelivery!) {
        msg.isDelivery = null; // When message create, set is is none;
      }
      this._messages[msg.id] = msg;
      setState(() {});
    }
  }

  _messageList(List params) {
    // TOOD load more history.
    params.forEach((param) {
        final msg = Message.fromList(param);
        this._messages[msg.id] = msg;
    });
    setState(() {});
  }

  _messageDelivery(List params) {
    final id = params[0];
    final isDelivery = params[1];
    if (this._messages.containsKey(id)) {
      this._messages[id]!.isDelivery = isDelivery;
      setState(() {});
    }
  }

  _loadFriend() async {
    this._messages.clear();
    final res = await httpPost('chat-detail', [widget.id]);
    if (res.isOk) {
      this._friend = Friend.fromList(res.params[0]);
      res.params[1].forEach((params) {
          final msg = Message.fromList(params);
          this._messages[msg.id] = msg;
      });
      setState(() { this._loading = false; });
    } else {
      print(res.error);
    }
  }

  _send(MessageType mtype, String raw) {
    rpc.send('chat-message-create', [_friend.id, _friend.gid, mtype.toInt(), raw]);
  }

  @override
  void deactivate() {
    if (!isDisplayDesktop(context)) {
      context.read<AccountProvider>().clearActivedSession(SessionType.Group);
    }
    super.deactivate();
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);
    // check change friend.
    if (this._friend.id != widget.id) {
      _loadFriend();
      setState(() { this._loading = true; });
    }

    final accountProvider = context.watch<AccountProvider>();
    final session = accountProvider.activedSession;
    final meName = accountProvider.activedAccount.name;
    final isOnline = session.isActive();

    final recentMessageKeys = this._messages.keys.toList().toList();

    return Scaffold(
      appBar: AppBar(
        automaticallyImplyLeading: false,
        leading: isDesktop ? null : IconButton(icon: Icon(Icons.arrow_back),
          onPressed: () => Navigator.pop(context)),
        title: Text(this._loading ? lang.waiting : _friend.name,
          maxLines: 1, overflow: TextOverflow.ellipsis),
        bottom: isDesktop ? PreferredSize(
          child: Container(color: const Color(0x40ADB0BB), height: 1.0),
          preferredSize: Size.fromHeight(1.0)): null,
        actions: [
          const SizedBox(width: 20.0),
          GestureDetector(
            onTap: () {},
            child: Container(
              width: 20.0,
              child: Icon(Icons.phone_rounded,
                color: Color(0x26ADB0BB))),
          ),
          const SizedBox(width: 20.0),
          GestureDetector(
            onTap: () {},
            child: Container(
              width: 20.0,
              child: Icon(Icons.videocam_rounded,
                color: Color(0x26ADB0BB))),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 20.0),
            child: PopupMenuButton<int>(
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(15)
              ),
              color: const Color(0xFFEDEDED),
              child: Icon(Icons.more_vert_rounded, color: color.primary),
              onSelected: (int value) {
                if (value == 0) {
                  showShadowDialog(
                    context,
                    Icons.info,
                    lang.friendInfo,
                    UserInfo(
                      app: 'add-friend',
                      id: _friend.gid,
                      name: _friend.name,
                      addr: _friend.addr,
                      title: lang.qrFriend,
                      remark: _friend.remark,
                    ),
                    0.0,
                  );
                } else if (value == 1) {
                  showShadowDialog(context, Icons.group, lang.groupChatAdd, GroupAddScreen(), 0.0);
                } else if (value == 2) {
                  print('TODO remark');
                } else if (value == 3) {
                  showDialog(
                    context: context,
                    builder: (BuildContext context) {
                      return AlertDialog(
                        title: Text(lang.unfriend),
                        content: Text(_friend.name,
                          style: TextStyle(color: color.primary)),
                        actions: [
                          TextButton(
                            child: Text(lang.cancel),
                            onPressed: () => Navigator.pop(context),
                          ),
                          TextButton(
                            child: Text(lang.ok),
                            onPressed:  () {
                              Navigator.pop(context);
                              rpc.send('chat-friend-close', [_friend.id]);
                              setState(() {
                                  this._friend.isClosed = true;
                              });
                            },
                          ),
                        ]
                      );
                    },
                  );
                } else if (value == 4) {
                  rpc.send('chat-request-create', [
                      _friend.gid, _friend.addr, _friend.name, lang.fromContactCard(meName)
                  ]);
                } else if (value == 5) {
                  showDialog(
                    context: context,
                    builder: (BuildContext context) {
                      return AlertDialog(
                        title: Text(lang.delete + " " + lang.friend),
                        content: Text(_friend.name,
                          style: TextStyle(color: Colors.red)),
                        actions: [
                          TextButton(
                            child: Text(lang.cancel),
                            onPressed: () => Navigator.pop(context),
                          ),
                          TextButton(
                            child: Text(lang.ok),
                            onPressed:  () {
                              Navigator.pop(context);
                              rpc.send('chat-friend-delete', [_friend.id]);
                              if (!isDesktop) {
                                Navigator.pop(context);
                              }
                            },
                          ),
                        ]
                      );
                    },
                  );
                }
              },
              itemBuilder: (context) {
                return <PopupMenuEntry<int>>[
                  menuItem(Color(0xFF6174FF), 0, Icons.qr_code_rounded, lang.friendInfo),
                  menuItem(Color(0xFF6174FF), 1, Icons.group_rounded, lang.groupChatAdd),
                  //_menuItem(color.primary, 2, Icons.turned_in_rounded, lang.remark),
                  _friend.isClosed
                  ? menuItem(Color(0xFF6174FF), 4, Icons.send_rounded, lang.addFriend)
                  : menuItem(Color(0xFF6174FF), 3, Icons.block_rounded, lang.unfriend),
                  menuItem(Colors.red, 5, Icons.delete_rounded, lang.delete),
                ];
              },
            )
          )
        ]
      ),
      body: Column(
        children: [
          Expanded(
            child: ListView.builder(
              padding: EdgeInsets.symmetric(horizontal: 20.0),
              itemCount: recentMessageKeys.length,
              reverse: true,
              itemBuilder: (BuildContext context, index) => ChatMessage(
                fgid: _friend.gid,
                name: _friend.name,
                message: this._messages[recentMessageKeys[index]]!,
              )
          )),
          if (!this._friend.isClosed)
          ChatInput(
            sid: session.id,
            online: isOnline,
            callback: _send,
            transferTo: '',
            waiting: session.online == OnlineType.Waiting
          ),
        ]
      )
    );
  }
}
