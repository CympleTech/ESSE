import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/utils/pick_image.dart';
import 'package:esse/utils/pick_file.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/avatar.dart';
import 'package:esse/widgets/emoji.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/audio_recorder.dart';
import 'package:esse/widgets/user_info.dart';
import 'package:esse/widgets/chat_message.dart';
import 'package:esse/widgets/chat_input.dart';
import 'package:esse/widgets/show_contact.dart';
import 'package:esse/rpc.dart';
import 'package:esse/global.dart';
import 'package:esse/provider.dart';
import 'package:esse/session.dart' show SessionType, Session;

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
  TextEditingController textController = TextEditingController();
  FocusNode textFocus = FocusNode();
  bool emojiShow = false;
  bool sendShow = false;
  bool menuShow = false;
  bool recordShow = false;
  String _recordName = '';

  bool _loading = false;
  GroupChat _group = GroupChat();
  List<Member> _members = [];
  List<Message> _messages = [];

  @override
  initState() {
    super.initState();
    textFocus.addListener(() {
        if (textFocus.hasFocus) {
          setState(() {
              emojiShow = false;
              menuShow = false;
              recordShow = false;
          });
        }
    });
  }

  _loadGroup() async {
    this._members.clear();
    this._messages.clear();
    final res = await httpPost('group-detail', [widget.id]);
    if (res.isOk) {
      this._group = GroupChat.fromList(res.params[0]);
      res.params[1].forEach((params) {
          this._members.add(Member.fromList(params));
      });
      res.params[2].forEach((params) {
          this._messages.add(Message.fromList(params));
      });
      setState(() { this._loading = false; });
    } else {
      print(res.error);
    }
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

    return Scaffold(
      key: _scaffoldKey,
      endDrawer: _MemberScreen(members: this._members),
      drawerScrimColor: const Color(0x26ADB0BB),
      appBar: AppBar(
        automaticallyImplyLeading: false,
        leading: reallyDesktop ? null : IconButton(icon: Icon(Icons.arrow_back),
          onPressed: () => Navigator.pop(context)),
        title: Text(this._loading ? lang.waiting : _group.name,
          maxLines: 1, overflow: TextOverflow.ellipsis),
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
            Expanded(child: _MainScreen(
                isDesktop: isDesktop, group: this._group, members: this._members, messages: this._messages)),
            _MemberScreen(members: this._members),
        ])
        : _MainScreen(isDesktop: isDesktop, group: this._group, members: this._members, messages: this._messages)
      )
    );
  }
}

class _MainScreen extends StatefulWidget {
  final bool isDesktop;
  final GroupChat group;
  final List<Member> members;
  final List<Message> messages;

  _MainScreen({Key? key, required this.isDesktop, required this.group, required this.members, required this.messages}) : super(key: key);

  @override
  _MainScreenState createState() => _MainScreenState();
}

class _MainScreenState extends State<_MainScreen> {
  @override
  void initState() {
    super.initState();
  }

  _send(MessageType mtype, String raw) {
    //
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Column(
      children: [
        Expanded(
          child: ListView.builder(
            padding: EdgeInsets.symmetric(horizontal: 20.0),
            itemCount: 0,
            reverse: true,
            itemBuilder: (BuildContext context, index) {
              return Container();
            }
        )),
        ChatInput(
          sid: 0,
          online: true,
          callback: _send,
          hasTransfer: false,
          emojiWidth: widget.isDesktop,
        ),
      ]
    );
  }
}

class _MemberScreen extends StatefulWidget {
  final List<Member> members;
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
        child: ListView(children: widget.members.map((member) => _item(member, lang)).toList())
    ));
  }
}
