import 'dart:ui' show ImageFilter;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/utils/toast.dart';
import 'package:esse/utils/pick_image.dart';
import 'package:esse/utils/pick_file.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/avatar.dart';
import 'package:esse/widgets/emoji.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/audio_recorder.dart';
import 'package:esse/widgets/user_info.dart';
import 'package:esse/widgets/chat_message.dart';
import 'package:esse/widgets/show_contact.dart';
import 'package:esse/rpc.dart';
import 'package:esse/global.dart';
import 'package:esse/provider.dart';
import 'package:esse/session.dart' show SessionType;

import 'package:esse/apps/primitives.dart';
import 'package:esse/apps/group_chat/models.dart';
import 'package:esse/apps/group_chat/provider.dart';

class GroupChatDetail extends StatefulWidget {
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

  @override
  void deactivate() {
    if (!isDisplayDesktop(context)) {
      context.read<AccountProvider>().clearActivedSession(SessionType.Group);
    }
    super.deactivate();
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
      context.read<GroupChatProvider>().messageCreate(MessageType.Image, image);
    }
    setState(() {
        emojiShow = false;
        sendShow = false;
        menuShow = false;
        recordShow = false;
    });
  }

  void _sendFile() async {
    final file = await pickFile();
    if (file != null) {
      context.read<GroupChatProvider>().messageCreate(MessageType.File, file);
    }
    setState(() {
        emojiShow = false;
        sendShow = false;
        menuShow = false;
        recordShow = false;
    });
  }

  void _sendRecord(int time) async {
    final raw = BaseMessage.rawRecordName(time, _recordName);
    if (raw != null) {
      context.read<GroupChatProvider>().messageCreate(MessageType.Record, raw);
    }
    setState(() {
        emojiShow = false;
        sendShow = false;
        menuShow = false;
        recordShow = false;
    });
  }

  _callback(int id) {
    if (id != null) {
      context.read<GroupChatProvider>().messageCreate(MessageType.Contact, "${id}");
    }
    setState(() {
        emojiShow = false;
        sendShow = false;
        menuShow = false;
        recordShow = false;
    });
  }

  void _sendContact(ColorScheme color, AppLocalizations lang) {
    showShadowDialog(
      context,
      Icons.person_rounded,
      lang.contact,
      ContactList(callback: _callback, multiple: false)
    );
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
                      provider.clearActivedGroup();
                      Navigator.pop(context);
                    },
                    child: Container(
                      width: 20.0,
                      child:
                      Icon(Icons.arrow_back, color: color.primary)),
                  ),
                  const SizedBox(width: 15.0),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          this.group.name,
                          style: TextStyle(fontWeight: FontWeight.bold),
                        ),
                        const SizedBox(height: 6.0),
                        Text(this.group.isClosed
                          ? lang.closed
                          : session.onlineLang(lang),
                          style: TextStyle(
                            color: color.onPrimary.withOpacity(0.5),
                            fontSize: 14.0))
                      ],
                    ),
                  ),
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
                  const SizedBox(width: 20.0),
                  GestureDetector(
                    onTap: () {
                      showShadowDialog(context, Icons.groups, this.group.name,
                        _MemberWidget(id: this.group.id, gid: this.group.gid),
                        0.0,
                      );
                    },
                    child: Container(
                      width: 20.0,
                      child: Icon(Icons.group_rounded, color: color.primary)),
                  ),
                  const SizedBox(width: 20.0),
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
                            addr: '0x' + this.group.addr,
                            title: this.group.type.lang(lang),
                            bio: this.group.bio,
                          ),
                          0.0,
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
                                    context.read<AccountProvider>().clearActivedSession(
                                      SessionType.Group
                                    );
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
                        _menuItem(Color(0xFF6174FF), 1, Icons.qr_code_rounded, lang.info),
                        if (!this.group.isClosed && isOnline)
                        _menuItem(Colors.orange, 2, Icons.block_rounded, lang.exit),
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
                padding: EdgeInsets.symmetric(horizontal: 10.0),
                itemCount: recentMessageKeys.length,
                reverse: true,
                itemBuilder: (BuildContext context, index) {
                  final message = recentMessages[recentMessageKeys[index]];
                  final member = members[message.mid];
                  return member == null ? ChatMessage(
                    fgid: "",
                    avatar: Avatar(name: lang.delete),
                    name: lang.delete,
                    message: message,
                  ) : ChatMessage(
                    fgid: member.mid,
                    avatar: member.showAvatar(isOnline: isOnline),
                    name: member.name,
                    message: message,
                  );
                }
            )),
            this.group.isClosed
            ? const SizedBox(height: 20.0)
            : Container(
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
                    action: () => _sendContact(color, lang),
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

class _MemberWidget extends StatelessWidget {
  final int id;
  final String gid;
  const _MemberWidget({Key key, this.id, this.gid}) : super(key: key);

  Widget _meItem(Member member, bool meOwner, bool meManager, Color color, lang) {
    return Container(
      height: 55.0,
      child: ListTile(
        leading: member.showAvatar(),
        title: Text(lang.me, textAlign: TextAlign.left,
          style: TextStyle(fontSize: 16.0, fontStyle: FontStyle.italic)),
        trailing: Text(meOwner ? lang.groupOwner : (meManager ? lang.manager : ''),
          style: TextStyle(color: color)),
      )
    );
  }

  Widget _item(context, bool isOnline, Member member, bool isOwner, bool meOwner, bool meManager, Color color, lang) {
    return Container(
      height: 55.0,
      child: ListTile(
        leading: member.showAvatar(isOnline: isOnline),
        title: Text(member.name, textAlign: TextAlign.left, style: TextStyle(fontSize: 16.0)),
        trailing: Text(member.isBlock
          ? lang.blocked : (isOwner
            ? lang.groupOwner : (member.isManager
              ? lang.manager : '')),
          style: TextStyle(color: color)),
        onTap: () {
          showShadowDialog(context, Icons.group_rounded, lang.members,
            MemberDetail(member: member, isGroupOwner: meOwner, isGroupManager: meManager),
            10.0,
          );
        }
      )
    );
  }

  _action(List<int> ids) {
    rpc.send('group-chat-invite', [id, gid, ids]);
  }

  _invite(context, String title) {
    Navigator.pop(context);
    showShadowDialog(context, Icons.people_rounded, title,
      ContactList(callback: _action, multiple: true),
      0.0,
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isLight = color.brightness == Brightness.light;
    final isDesktop = isDisplayDesktop(context);
    final accountProvider = context.read<AccountProvider>();
    final myId = accountProvider.activedAccountId;
    final isOnline = accountProvider.activedSession.isActive();

    final provider = context.watch<GroupChatProvider>();
    final members = provider.activedMembers;
    final all = provider.activedMemberOrder(myId);
    final allKeys = all[0];
    final meId = all[1];
    final meOwner = all[2];
    final meManager = all[3];

    double maxHeight = MediaQuery.of(context).size.height - 400;
    if (maxHeight < 100.0) {
      maxHeight = 100.0;
    }

    return Column(
      children: [
        Container(
          alignment: Alignment.centerRight,
          child: Text("${lang.members} (${allKeys.length + 1})",
            style: Theme.of(context).textTheme.headline6),
        ),
        const SizedBox(height: 10.0),
        const Divider(height: 1.0, color: Color(0x40ADB0BB)),
        const SizedBox(height: 10.0),
        _meItem(members[meId], meOwner, meManager, color.primary, lang),
        Container(
          height: maxHeight,
          child: SingleChildScrollView(
            child: Column(children: List<Widget>.generate(allKeys.length, (i) =>
                _item(
                  context, isOnline, members[allKeys[i]],
                  i == 0 && !meOwner, meOwner, meManager, color.primary, lang
                ),
            ))
          )
        ),
        const SizedBox(height: 10.0),
        const Divider(height: 1.0, color: Color(0x40ADB0BB)),
        const SizedBox(height: 10.0),
        TextButton(child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(Icons.add, size: 16.0),
              Text(lang.invite, style: TextStyle(fontSize: 14.0)),
            ]
          ),
          onPressed: () => _invite(context, lang.contact)
        ),
    ]);
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
        if (widget.isGroupOwner)
        Container(
          padding: const EdgeInsets.only(top: 20.0, bottom: 10.0),
          child: InkWell(
            onTap: () {
              Navigator.pop(context);
              // TODO delete.
            },
            hoverColor: Colors.transparent,
            child: Container(
              width: 300.0,
              padding: const EdgeInsets.symmetric(vertical: 10.0),
              decoration: BoxDecoration(
                border: Border.all(color: color.primary),
                borderRadius: BorderRadius.circular(10.0)),
              child: Center(child: Text(widget.member.isManager ? 'Cancel Manager' : 'Set Manager',
                  style: TextStyle(fontSize: 14.0, color: color.primary))),
            )
          )
        ),
        if (notFriend)
        Container(
          padding: const EdgeInsets.symmetric(vertical: 10.0),
          child: InkWell(
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
          )
        ),
        if (widget.isGroupManager || widget.isGroupOwner)
        Container(
          padding: const EdgeInsets.symmetric(vertical: 10.0),
          child: InkWell(
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
          )
        ),
        Container(
          padding: const EdgeInsets.symmetric(vertical: 10.0),
          child: InkWell(
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
                border: Border.all(color: Colors.black),
                borderRadius: BorderRadius.circular(10.0)),
              child: Center(child: Text(widget.member.isBlock ? lang.blocked : lang.block,
                  style: TextStyle(fontSize: 14.0, color: Colors.black))),
            )
          )
        ),
      ]
    );
  }
}
