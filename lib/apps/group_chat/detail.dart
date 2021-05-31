import 'dart:ui' show ImageFilter;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/utils/toast.dart';
import 'package:esse/utils/pick_image.dart';
import 'package:esse/utils/pick_file.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/emoji.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/audio_recorder.dart';
import 'package:esse/widgets/user_info.dart';
import 'package:esse/widgets/chat_message.dart';
import 'package:esse/global.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/primitives.dart';
import 'package:esse/apps/chat/provider.dart';
import 'package:esse/apps/group_chat/models.dart';
import 'package:esse/apps/group_chat/provider.dart';

class GroupChatDetail extends StatefulWidget {
  static GlobalKey<ScaffoldState> _scaffoldKey = new GlobalKey<ScaffoldState>();

  const GroupChatDetail({Key key}) : super(key: key);

  @override
  _GroupChatDetailState createState() => _GroupChatDetailState();
}

class _GroupChatDetailState extends State<GroupChatDetail> {
  TextEditingController textController = TextEditingController();
  FocusNode textFocus = FocusNode();
  bool emojiShow = false;
  bool sendShow = false;
  bool menuShow = false;
  bool recordShow = false;
  String _recordName;

  GroupChat group;

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

  _generateRecordPath() {
    this._recordName = DateTime.now().millisecondsSinceEpoch.toString() +
    '_' +
    this.group.id.toString() +
    '.m4a';
  }

  void _sendMessage() async {
    if (textController.text.length < 1) {
      return;
    }

    context.read<GroupChatProvider>().messageCreate(MessageType.String, textController.text);
    setState(() {
        textController.text = '';
        textFocus.requestFocus();

        emojiShow = false;
        sendShow = false;
        menuShow = false;
        recordShow = false;
    });
  }

  void _selectEmoji(value) {
    textController.text += value;
  }

  void _sendImage() async {
    final image = await pickImage();
    if (image != null) {
      //context.read<GroupChatProvider>().messageCreate(Message(group.id, MessageType.Image, image));
    }
    setState(() {
        textFocus.requestFocus();
        emojiShow = false;
        sendShow = false;
        menuShow = false;
        recordShow = false;
    });
  }

  void _sendFile() async {
    final file = await pickFile();
    if (file != null) {
      //context.read<GroupChatProvider>().messageCreate(Message(group.id, MessageType.File, file));
    }
    setState(() {
        textFocus.requestFocus();
        emojiShow = false;
        sendShow = false;
        menuShow = false;
        recordShow = false;
    });
  }

  void _sendRecord(int time) async {
    final raw = BaseMessage.rawRecordName(time, _recordName);
    //context.read<GroupChatProvider>().messageCreate(Message(group.id, MessageType.Record, raw));

    setState(() {
        textFocus.requestFocus();
        emojiShow = false;
        sendShow = false;
        menuShow = false;
        recordShow = false;
    });
  }

  void _sendContact(ColorScheme color, AppLocalizations lang, friends) {
    showShadowDialog(
      context,
      Icons.person_rounded,
      'Contact',
      Column(children: [
          Container(
            height: 40.0,
            decoration: BoxDecoration(
              color: color.surface,
              borderRadius: BorderRadius.circular(15.0)),
            child: TextField(
              autofocus: false,
              textInputAction: TextInputAction.search,
              textAlignVertical: TextAlignVertical.center,
              style: TextStyle(fontSize: 14.0),
              onSubmitted: (value) {
                toast(context, 'WIP...');
              },
              decoration: InputDecoration(
                hintText: lang.search,
                hintStyle: TextStyle(color: color.onPrimary.withOpacity(0.5)),
                border: InputBorder.none,
                contentPadding:
                EdgeInsets.only(left: 15.0, right: 15.0, bottom: 15.0),
              ),
            ),
          ),
          SizedBox(height: 15.0),
          Column(
            children: friends.map<Widget>((contact) {
                return GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onTap: () async {
                    //context.read<GroupChatProvider>().messageCreate(Message(friend.id, MessageType.Contact, "${contact.id}"));
                    Navigator.of(context).pop();
                    setState(() {
                        textFocus.requestFocus();
                        emojiShow = false;
                        sendShow = false;
                        menuShow = false;
                        recordShow = false;
                    });
                  },
                  child: Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 20.0, vertical: 14.0),
                    child: Row(
                      children: [
                        contact.showAvatar(),
                        SizedBox(width: 15.0),
                        Text(contact.name, style: TextStyle(fontSize: 16.0)),
                      ],
                    ),
                  ),
                );
            }).toList())
    ]));
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);

    final provider = context.watch<GroupChatProvider>();
    final members = provider.activedMembers;
    final recentMessages = provider.activedMessages;
    final recentMessageKeys = recentMessages.keys.toList().reversed.toList();

    this.group = provider.activedGroup;
    final isGroupOwner = provider.isActivedGroupOwner;
    final isGroupManager = provider.isActivedGroupManager;

    if (this.group == null) {
      return Container(
        padding: EdgeInsets.only(left: 20.0, right: 20.0, top: 10.0, bottom: 10.0),
        child: Text('Waiting...')
      );
    }

    final accountProvider = context.watch<AccountProvider>();
    final session = accountProvider.activedSession;
    final meName = accountProvider.activedAccount.name;
    final isOnline = session.isActive();

    return Scaffold(
      key: GroupChatDetail._scaffoldKey,
      endDrawer: _MemberDrawerWidget(title: lang.members),
      drawerScrimColor: color.background,
      body: SafeArea(
        child: Column(
          children: [
            Container(
              padding: EdgeInsets.only(left: 20.0, right: 20.0, top: 10.0, bottom: 10.0),
              child: Row(
                children: [
                  if (!isDesktop)
                  GestureDetector(
                    onTap: () {
                      context.read<GroupChatProvider>().clearActivedGroup();
                      Navigator.pop(context);
                    },
                    child: Container(
                      width: 20.0,
                      child:
                      Icon(Icons.arrow_back, color: color.primary)),
                  ),
                  SizedBox(width: 15.0),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          this.group.name,
                          style: TextStyle(fontWeight: FontWeight.bold),
                        ),
                        SizedBox(height: 6.0),
                        Text(this.group.isClosed
                          ? lang.unfriended
                          : session.onlineLang(lang),
                          style: TextStyle(
                            color: color.onPrimary.withOpacity(0.5),
                            fontSize: 14.0))
                      ],
                    ),
                  ),
                  SizedBox(width: 20.0),
                  GestureDetector(
                    onTap: () {},
                    child: Container(
                      width: 20.0,
                      child: Icon(Icons.phone_rounded,
                        color: Color(0x26ADB0BB))),
                  ),
                  SizedBox(width: 20.0),
                  GestureDetector(
                    onTap: () {},
                    child: Container(
                      width: 20.0,
                      child: Icon(Icons.videocam_rounded,
                        color: Color(0x26ADB0BB))),
                  ),
                  SizedBox(width: 20.0),
                  GestureDetector(
                    onTap: () {
                      GroupChatDetail._scaffoldKey.currentState.openEndDrawer();
                    },
                    child: Container(
                      width: 20.0,
                      child: Icon(Icons.group_rounded, color: color.primary)),
                  ),
                  SizedBox(width: 20.0),
                  PopupMenuButton<int>(
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(15)
                    ),
                    color: const Color(0xFFEDEDED),
                    child: Icon(Icons.more_vert_rounded, color: color.primary),
                    onSelected: (int value) {
                      if (value == 1) {
                        showShadowDialog(context, Icons.info, lang.groupChat,
                          UserInfo(
                            app: 'add-group',
                            id: 'EG' + this.group.gid.toUpperCase(),
                            name: this.group.name,
                            addr: '0x' + this.group.addr
                          )
                        );
                      } else if (value == 2) {
                        showDialog(
                          context: context, builder: (BuildContext context) {
                            return AlertDialog(
                              title: Text(lang.exit),
                              content: Text(this.group.name,
                                style: TextStyle(color: color.primary)),
                              actions: [
                                TextButton(child: Text(lang.cancel),
                                  onPressed: () => Navigator.pop(context),
                                ),
                                TextButton(child: Text(lang.ok),
                                  onPressed:  () {
                                    Navigator.pop(context);
                                    provider.close(this.group.id);
                                  },
                                ),
                              ]
                            );
                          },
                        );
                      } else if (value == 3) {
                        showDialog(
                          context: context, builder: (BuildContext context) {
                            return AlertDialog(
                              title: Text(lang.delete + ' ' + lang.groupChat),
                              content: Text(this.group.name,
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
                                    provider.delete(this.group.id);
                                  },
                                ),
                              ]
                            );
                          },
                        );
                      } else if (value == 4) {
                        provider.reAdd(this.group.id);
                      }
                    },
                    itemBuilder: (context) {
                      return <PopupMenuEntry<int>>[
                        _menuItem(Color(0xFF6174FF), 1, Icons.qr_code_rounded, lang.info),
                        this.group.isClosed
                        ? _menuItem(Color(0xFF6174FF), 4, Icons.send_rounded, lang.addGroup)
                        : _menuItem(Colors.orange, 2, Icons.block_rounded, lang.exit),
                        _menuItem(Colors.red, 3, Icons.delete_rounded, lang.delete),
                      ];
                    },
                  )
                ]
              ),
            ),
            const Divider(height: 1.0, color: Color(0x40ADB0BB)),
            Expanded(
              child: ListView.builder(
                padding: EdgeInsets.symmetric(horizontal: 20.0),
                itemCount: recentMessageKeys.length,
                reverse: true,
                itemBuilder: (BuildContext context, index) {
                  final message = recentMessages[recentMessageKeys[index]];
                  final member = members[message.fid];
                  return ChatMessage(
                    avatar: member.showAvatar(),
                    name: member.name,
                    message: message,
                  );
                }
            )),
            if (!this.group.isClosed)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 10.0),
              child: Row(
                children: [
                  GestureDetector(
                    onTap: isOnline ? () async {
                      if (recordShow) {
                        recordShow = false;
                        textFocus.requestFocus();
                      } else {
                        _generateRecordPath();
                        setState(() {
                            menuShow = false;
                            emojiShow = false;
                            recordShow = true;
                            textFocus.unfocus();
                        });
                      }
                    } : null,
                    child: Container(
                      width: 20.0,
                      child: Icon(Icons.mic_rounded, color: isOnline ? color.primary : Color(0xFFADB0BB))),
                  ),
                  SizedBox(width: 10.0),
                  Expanded(
                    child: Container(
                      height: 40,
                      decoration: BoxDecoration(
                        color: color.surface,
                        borderRadius: BorderRadius.circular(15.0),
                      ),
                      child: TextField(
                        enabled: isOnline,
                        style: TextStyle(fontSize: 14.0),
                        textInputAction: TextInputAction.send,
                        onChanged: (value) {
                          if (value.length == 0 && sendShow) {
                            setState(() {
                                sendShow = false;
                            });
                          } else {
                            if (!sendShow) {
                              setState(() {
                                  sendShow = true;
                              });
                            }
                          }
                        },
                        onSubmitted: (_v) => _sendMessage(),
                        decoration: InputDecoration(
                          hintText: 'Aa',
                          border: InputBorder.none,
                          contentPadding: EdgeInsets.only(
                            left: 15.0, right: 15.0, bottom: 7.0),
                        ),
                        controller: textController,
                        focusNode: textFocus,
                      ),
                    ),
                  ),
                  SizedBox(width: 10.0),
                  GestureDetector(
                    onTap: isOnline ? () {
                      if (emojiShow) {
                        textFocus.requestFocus();
                      } else {
                        setState(() {
                            menuShow = false;
                            recordShow = false;
                            emojiShow = true;
                            textFocus.unfocus();
                        });
                      }
                    } : null,
                    child: Container(
                      width: 20.0,
                      child: Icon(
                        emojiShow
                        ? Icons.keyboard_rounded
                        : Icons.emoji_emotions_rounded,
                        color: isOnline ? color.primary : Color(0xFFADB0BB))),
                  ),
                  SizedBox(width: 10.0),
                  sendShow
                  ? GestureDetector(
                    onTap: isOnline ? _sendMessage : null,
                    child: Container(
                      width: 50.0,
                      height: 30.0,
                      decoration: BoxDecoration(
                        color: Color(0xFF6174FF),
                        borderRadius: BorderRadius.circular(10.0),
                      ),
                      child: Center(
                        child: Icon(Icons.send,
                          color: Colors.white, size: 20.0))),
                  )
                  : GestureDetector(
                    onTap: isOnline ? () {
                      if (menuShow) {
                        textFocus.requestFocus();
                      } else {
                        setState(() {
                            emojiShow = false;
                            recordShow = false;
                            menuShow = true;
                            textFocus.unfocus();
                        });
                      }
                    }  : null,
                    child: Container(
                      width: 20.0,
                      child: Icon(Icons.add_circle_rounded,
                        color: isOnline ? color.primary : Color(0xFFADB0BB))),
                  ),
                ],
              ),
            ),
            if (emojiShow && isOnline) Emoji(action: _selectEmoji),
            if (recordShow && isOnline)
            Container(
              height: 100.0,
              child: AudioRecorder(
                path: Global.recordPath + _recordName, onStop: _sendRecord),
            ),
            if (menuShow && isOnline)
            Container(
              height: 100.0,
              child: Wrap(
                spacing: 20.0,
                runSpacing: 20.0,
                alignment: WrapAlignment.center,
                children: <Widget>[
                  ExtensionButton(
                    icon: Icons.image_rounded,
                    text: lang.album,
                    action: _sendImage,
                    bgColor: color.surface,
                    iconColor: color.primary),
                  ExtensionButton(
                    icon: Icons.folder_rounded,
                    text: lang.file,
                    action: _sendFile,
                    bgColor: color.surface,
                    iconColor: color.primary),
                  ExtensionButton(
                    icon: Icons.person_rounded,
                    text: lang.contact,
                    action: () => _sendContact(color, lang, context.read<ChatProvider>().friends.values),
                    bgColor: color.surface,
                    iconColor: color.primary),
                ],
              ),
            )
          ],
        )
      )
    );
  }
}

class ExtensionButton extends StatelessWidget {
  final String text;
  final IconData icon;
  final Function action;
  final Color bgColor;
  final Color iconColor;

  const ExtensionButton({
      Key key,
      this.icon,
      this.text,
      this.action,
      this.bgColor,
      this.iconColor,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: action,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Container(
            padding: const EdgeInsets.all(10.0),
            decoration: BoxDecoration(
              color: bgColor,
              borderRadius: BorderRadius.circular(15.0),
            ),
            child: Icon(icon, color: iconColor, size: 36.0)),
          SizedBox(height: 5.0),
          Text(text, style: TextStyle(fontSize: 14.0)),
        ],
    ));
  }
}

Widget _menuItem(Color color, int value, IconData icon, String text) {
  return PopupMenuItem<int>(
    value: value,
    child: Row(
      children: [
        Icon(icon, color: color),
        Padding(
          padding: const EdgeInsets.only(left: 20.0, right: 10.0),
          child: Text(text, style: TextStyle(color: Colors.black, fontSize: 16.0)),
        )
      ]
    ),
  );
}

class _MemberDrawerWidget extends StatelessWidget {
  final String title;
  const _MemberDrawerWidget({Key key, this.title}) : super(key: key);

  Widget _item(context, Member member, bool isOwner, Color color) {
    return Container(
      height: 55.0,
      child: ListTile(
        leading: member.showAvatar(colorSurface: false),
        title: Text(member.name, textAlign: TextAlign.left, style: TextStyle(fontSize: 16.0)),
        trailing: Text(member.isBlock
          ? 'Blocked' : (isOwner
            ? 'Owner' : (member.isManager
              ? 'Manager' : '')),
          style: TextStyle(color: color)),
        onTap: () {
          Navigator.pop(context);
          showShadowDialog(context, Icons.group_rounded, title,
            MemberDetail(member: member, isGroupOwner: isOwner, isGroupManager: member.isManager),
            10.0,
          );
        }
      )
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isLight = color.brightness == Brightness.light;
    final isDesktop = isDisplayDesktop(context);

    final provider = context.watch<GroupChatProvider>();
    final members = provider.activedMembers;
    final allKeys = provider.activedMemberOrder;

    return Drawer(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 4.0, sigmaY: 4.0),
        child: SafeArea(
          child: Container(
            decoration: BoxDecoration(color: color.surface),
            padding: const EdgeInsets.symmetric(vertical: 20.0),
            child: Column(
              children: [
                Text(lang.members, style: Theme.of(context).textTheme.title),
                const SizedBox(height: 10.0),
                const Divider(height: 1.0, color: Color(0x40ADB0BB)),
                const SizedBox(height: 10.0),
                Expanded(
                  child: ListView.builder(
                    itemCount: allKeys.length,
                    itemBuilder: (BuildContext ctx, int index) => _item(
                      context, members[allKeys[index]], index == 0, color.primary
                    ),
                  )
                )
              ]
            ),
          )
        )
      )
    );
  }
}

class MemberDetail extends StatefulWidget {
  Member member;
  bool isGroupManager;
  bool isGroupOwner;

  MemberDetail({Key key, this.member, this.isGroupManager, this.isGroupOwner}) : super(key: key);

  @override
  _MemberDetailState createState() => _MemberDetailState();
}

class _MemberDetailState extends State<MemberDetail> {
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

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final bool notFriend = false; // TODO

    return Column(
      mainAxisSize: MainAxisSize.max,
      children: [
        widget.member.showAvatar(width: 100.0),
        const SizedBox(height: 10.0),
        Text(widget.member.name),
        const SizedBox(height: 10.0),
        const Divider(height: 1.0, color: Color(0x40ADB0BB)),
        const SizedBox(height: 10.0),
        _infoListTooltip(Icons.person, color.primary, widget.member.mid),
        _infoListTooltip(Icons.location_on, color.primary, widget.member.addr),
        const SizedBox(height: 10.0),
        if (widget.isGroupOwner)
        InkWell(
          onTap: () {
            Navigator.pop(context);
            // TODO delete.
          },
          hoverColor: Colors.transparent,
          child: Container(
            width: 300.0,
            padding: const EdgeInsets.symmetric(vertical: 10.0),
            decoration: BoxDecoration(
              border: Border.all(),
              borderRadius: BorderRadius.circular(10.0)),
            child: Center(child: Text(widget.member.isManager ? 'Cancel Manager' : 'Set Manager',
                style: TextStyle(fontSize: 14.0))),
          )
        ),
        const SizedBox(height: 10.0),
        if (notFriend)
        InkWell(
          onTap: () {
            Navigator.pop(context);
            // TODO delete.
          },
          hoverColor: Colors.transparent,
          child: Container(
            width: 300.0,
            padding: const EdgeInsets.symmetric(vertical: 10.0),
            decoration: BoxDecoration(
              border: Border.all(color: Color(0xFF6174FF)),
              borderRadius: BorderRadius.circular(10.0)),
            child: Center(child: Text(lang.addFriend,
                style: TextStyle(fontSize: 14.0, color: Color(0xFF6174FF)))),
          )
        ),
        const SizedBox(height: 10.0),
        if (widget.isGroupManager || widget.isGroupOwner)
        InkWell(
          onTap: () {
            Navigator.pop(context);
            // TODO delete.
          },
          hoverColor: Colors.transparent,
          child: Container(
            width: 300.0,
            padding: const EdgeInsets.symmetric(vertical: 10.0),
            decoration: BoxDecoration(
              border: Border.all(color: Colors.red),
              borderRadius: BorderRadius.circular(10.0)),
            child: Center(child: Text(lang.delete,
                style: TextStyle(fontSize: 14.0, color: Colors.red))),
          )
        ),
        InkWell(
          onTap: () {
            Navigator.pop(context);
            context.read<GroupChatProvider>().memberUpdate(
              widget.member.id, !widget.member.isBlock);
          },
          hoverColor: Colors.transparent,
          child: Container(
            width: 300.0,
            padding: const EdgeInsets.symmetric(vertical: 10.0),
            decoration: BoxDecoration(
              border: Border.all(color: Colors.red),
              borderRadius: BorderRadius.circular(10.0)),
            child: Center(child: Text(widget.member.isBlock ? 'Blocked' : 'Block',
                style: TextStyle(fontSize: 14.0, color: Colors.red))),
          )
        ),
      ]
    );
  }
}
