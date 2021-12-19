import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/avatar.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/user_info.dart';
import 'package:esse/widgets/chat_message.dart';
import 'package:esse/widgets/chat_input.dart';
import 'package:esse/rpc.dart';
import 'package:esse/global.dart';
import 'package:esse/provider.dart';
import 'package:esse/session.dart' show SessionType, Session, OnlineType;

import 'package:esse/apps/primitives.dart';
import 'package:esse/apps/group/models.dart';

class GroupChatDetail extends StatefulWidget {
  final int id;
  GroupChatDetail({Key? key, required this.id}) : super(key: key);

  @override
  _GroupChatDetailState createState() => _GroupChatDetailState();
}

class _GroupChatDetailState extends State<GroupChatDetail> {
  final GlobalKey<ScaffoldState> _scaffoldKey = GlobalKey<ScaffoldState>();

  bool _loading = false;
  GroupChat _group = GroupChat();
  Map<int, Member> _members = {};
  Map<int, Message> _messages = {};

  @override
  initState() {
    super.initState();

    rpc.addListener('group-member-join', _memberJoin, false);
    rpc.addListener('group-member-leave', _memberLeave, false);
    rpc.addListener('group-member-online', _memberOnline, false);
    rpc.addListener('group-member-offline', _memberOffline, false);
    rpc.addListener('group-message-create', _messageCreate, false);
    rpc.addListener('group-message-delivery', _messageDelivery, false);
  }

  // [group, [member], [message]]
  _loadGroup() async {
    this._members.clear();
    this._messages.clear();
    final res = await httpPost('group-detail', [widget.id]);
    if (res.isOk) {
      this._group = GroupChat.fromList(res.params[0]);
      res.params[1].forEach((params) {
          this._members[params[0]] = Member.fromList(params);
      });
      res.params[2].forEach((params) {
          this._messages[params[0]] = Message.fromList(params);
      });
      setState(() { this._loading = false; });
    } else {
      print(res.error);
    }
  }

  // [member]
  _memberJoin(List params) {
    final member = Member.fromList(params);
    if (_group.id == member.fid) {
      this._members[member.id] = member;
      // TODO Better add UI member joined.
      setState(() {});
    }
  }

  // [group_id, member_id]
  _memberLeave(List params) {
    if (_group.id == params[0]) {
      this._members.remove(params[1]);
      setState(() {});
    }
  }

  // [group_id, member_id, member_addr]
  _memberOnline(List params) {
    if (_group.id == params[0] && this._members.containsKey(params[1])) {
      this._members[params[1]]!.addr = params[2];
      this._members[params[1]]!.online = true;
      setState(() {});
    }
  }

  // [group_id, member_id]
  _memberOffline(List params) {
    if (_group.id == params[0] && this._members.containsKey(params[1])) {
      this._members[params[1]]!.online = false;
      setState(() {});
    }
  }

  // [message]
  _messageCreate(List params) {
    Message msg = Message.fromList(params);
    if (_group.id == msg.fid) {
      if (!msg.isDelivery!) {
        msg.isDelivery = null;
      }
      this._messages[msg.id] = msg;
      setState(() {});
    }
  }

  // [message_id, is_delivery]
  _messageDelivery(List params) {
    final id = params[0];
    final isDelivery = params[1];
    if (this._messages.containsKey(id)) {
      this._messages[id]!.isDelivery = isDelivery;
      setState(() {});
    }
  }

  _send(MessageType mtype, String raw) {
    rpc.send('group-message-create', [_group.id, mtype.toInt(), raw]);
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
    final reallyDesktop = isDisplayDesktop(context);
    // check change group.
    if (this._group.id != widget.id) {
      _loadGroup();
      setState(() { this._loading = true; });
    }

    final width = MediaQuery.of(context).size.width;
    bool isDesktop = true;
    if (width - 520 < 500) {
      isDesktop = false;
    }

    final accountProvider = context.watch<AccountProvider>();
    final session = accountProvider.activedSession;
    final meName = accountProvider.activedAccount.name;
    final isOnline = session.isActive();
    final recentMessageKeys = this._messages.keys.toList().reversed.toList();

    return Scaffold(
      key: _scaffoldKey,
      endDrawer: _MemberScreen(members: this._members),
      drawerScrimColor: const Color(0x26ADB0BB),
      appBar: AppBar(
        automaticallyImplyLeading: false,
        leading: reallyDesktop ? null : IconButton(icon: Icon(Icons.arrow_back),
          onPressed: () => Navigator.pop(context)),
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(this._loading ? lang.waiting : _group.name,
              maxLines: 1, overflow: TextOverflow.ellipsis),
            const SizedBox(height: 2.0),
            Text(this._group.isClosed
              ? lang.closed
              : session.onlineLang(lang),
              style: TextStyle(color: color.primary, fontSize: 11.0))
          ]
        ),
        bottom: isDesktop ? PreferredSize(
          child: Container(color: const Color(0x40ADB0BB), height: 1.0),
          preferredSize: Size.fromHeight(1.0)): null,
        actions: [
          if (!isDesktop)
          IconButton(icon: Icon(Icons.people),
            onPressed: () => _scaffoldKey.currentState!.openEndDrawer(),
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
                } else if (value == 1) {
                } else if (value == 2) {
                }
              },
              itemBuilder: (context) {
                return <PopupMenuEntry<int>>[
                  menuItem(Color(0xFF6174FF), 0, Icons.add_rounded, lang.addFriend),
                  menuItem(Color(0xFF6174FF), 0, Icons.create_rounded, lang.rename),
                  menuItem(Colors.red, 6, Icons.delete_rounded, lang.delete),
                ];
              },
            )
          )
        ]
      ),
      body: Container(
        alignment: Alignment.topCenter,
        child: isDesktop
        ? Row(children: [
            Expanded(child: _mainScreen(isDesktop, session.id, isOnline, session.online == OnlineType.Waiting, recentMessageKeys)),
            _MemberScreen(members: this._members),
        ])
        : _mainScreen(isDesktop, session.id, isOnline, session.online == OnlineType.Waiting, recentMessageKeys)
      )
    );
  }

  Widget _mainScreen(isDesktop, sid, isOnline, waiting, recentMessageKeys) {
    return Column(
      children: [
        Expanded(
          child: ListView.builder(
            padding: EdgeInsets.symmetric(horizontal: 20.0),
            itemCount: recentMessageKeys.length,
            reverse: true,
            itemBuilder: (BuildContext context, index) => ChatMessage(
              fgid: this._group.gid,
              name: this._group.name,
              message: this._messages[recentMessageKeys[index]]!,
            )
        )),
        ChatInput(
          sid: sid,
          online: isOnline,
          callback: _send,
          hasTransfer: false,
          emojiWidth: isDesktop,
          waiting: waiting,
        ),
      ]
    );
  }
}

class _MemberScreen extends StatefulWidget {
  final Map<int, Member> members;
  _MemberScreen({Key? key, required this.members}) : super(key: key);

  @override
  _MemberScreenState createState() => _MemberScreenState();
}

class _MemberScreenState extends State<_MemberScreen> {
  @override
  void initState() {
    super.initState();
  }

  Widget _item(Member member, lang) {
    return Container(
      height: 60.0,
      child: ListTile(
        leading: member.showAvatar(),
        title: Text(member.name, textAlign: TextAlign.left, style: TextStyle(fontSize: 16.0)),
        onTap: () => showShadowDialog(
          context,
          Icons.info,
          lang.friendInfo,
          UserInfo(
            app: 'add-friend',
            id: member.mid,
            name: member.name,
            addr: member.addr,
            title: lang.qrFriend,
          ),
          0.0,
        ),
      )
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return SafeArea(
      child: Container(
        width: 200.0,
        padding: const EdgeInsets.symmetric(vertical: 20.0),
        decoration: BoxDecoration(color: color.secondary),
        child: ListView(children: widget.members.values.map((member) => _item(member, lang)).toList())
    ));
  }
}
