import 'dart:io';

import 'package:flutter/material.dart';
import 'package:image_save/image_save.dart';
import 'package:provider/provider.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:open_file/open_file.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/file_image.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/widgets/avatar.dart';
import 'package:esse/widgets/audio_player.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/user_info.dart';
import 'package:esse/global.dart';

import 'package:esse/apps/primitives.dart';
import 'package:esse/apps/chat/models.dart' show Request;
import 'package:esse/apps/group_chat/models.dart' show GroupType, GroupTypeExtension;
import 'package:esse/apps/chat/provider.dart';
import 'package:esse/apps/group_chat/provider.dart';

class ChatMessage extends StatelessWidget {
  final Widget? avatar;
  final String fgid;
  final String name;
  final BaseMessage message;

  const ChatMessage({Key? key, required this.fgid, required this.name, required this.message, this.avatar}): super(key: key);

  Widget _showContactCard(Widget avatar, String gid, String name, String title, ColorScheme color) {
    return Container(
      padding: const EdgeInsets.only(top: 10, bottom: 6.0, left: 10.0, right: 10.0),
      width: 200.0,
      decoration: BoxDecoration(color: const Color(0x40ADB0BB), borderRadius: BorderRadius.circular(10.0)),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(children: [
              avatar,
              Container(width: 135.0, padding: const EdgeInsets.only(left: 10.0),
                child: Column(children: [
                    Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: TextStyle(color: color.onPrimary, fontSize: 16.0)),
                    const SizedBox(height: 4.0),
                    Text(betterPrint(gid), style: TextStyle(color: Colors.grey, fontSize: 12.0)),
          ]))]),
          const SizedBox(height: 5.0),
          const Divider(height: 1.0, color: Color(0x40ADB0BB)),
          const SizedBox(height: 3.0),
          Text(title, style: TextStyle(color: Colors.grey, fontSize: 10.0)),
      ])
    );
  }

  Widget _showText(context, color, maxWidth) {
    // text
    return Container(
      constraints: BoxConstraints(minWidth: 50, maxWidth: maxWidth),
      padding: const EdgeInsets.symmetric(vertical: 6.0, horizontal: 10.0),
      decoration: BoxDecoration(
        color: message.isMe ? Color(0xFF6174FF) : color.primaryVariant,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: Text(message.content,
        style: TextStyle(
          color: message.isMe ? Colors.white : Color(0xFF1C1939),
          fontSize: 14.0)));
  }

  Widget _showImage(context, lang, color) {
    // image
    bool imageExsit = true;
    var thumImage;
    final imagePath = Global.imagePath + message.content;
    final thumPath = Global.thumbPath + message.content;
    if (FileSystemEntity.typeSync(thumPath) ==
      FileSystemEntityType.notFound) {
      imageExsit = false;
      thumImage = AssetImage('assets/images/image_missing.png');
    } else {
      thumImage = FileImage(File(thumPath));
    }
    return GestureDetector(
      onTap: imageExsit
      ? () => showShadowDialog(
        context,
        Icons.image_rounded,
        lang.album,
        Column(children: [
            Image(image: FileImage(File(imagePath)), fit: BoxFit.cover),
            SizedBox(height: 15.0),
            if (Platform.isAndroid || Platform.isIOS)
            InkWell(
              onTap: () async {
                Map<Permission, PermissionStatus> statuses = await [
                  Permission.storage,
                ].request();

                if (statuses[Permission.storage] == PermissionStatus.granted) {

                  // Save to album.
                  final data = await File(imagePath).readAsBytes();
                  final bool? success = await ImageSave.saveImage(data, message.content, albumName: "ESSE");
                  print(success);

                  Navigator.pop(context);
                }
              },
              hoverColor: Colors.transparent,
              child: Container(
                width: 200.0,
                padding: const EdgeInsets.symmetric(vertical: 10.0),
                decoration: BoxDecoration(
                  border: Border.all(color: color.primary),
                  borderRadius: BorderRadius.circular(10.0)),
                child: Center(child: Text(lang.download,
                    style: TextStyle(fontSize: 14.0, color: color.primary))),
              )
            ),
      ]))
      : () => {},
      child: Container(
        width: imageExsit ? 120.0 : 60.0,
        child: Image(image: thumImage, fit: BoxFit.cover),
    ));
  }

  Widget _showFile(context, lang, color) {
    // file
    bool fileExsit = true;
    Widget fileImage;
    final filePath = Global.filePath + message.content;
    if (FileSystemEntity.typeSync(filePath) ==
      FileSystemEntityType.notFound) {
      fileExsit = false;
      fileImage = Image(image: AssetImage('assets/images/image_missing.png'), fit: BoxFit.cover);
    } else {
      fileImage = fileIcon(message.content, 36.0);
    }
    return GestureDetector(
      onTap: fileExsit
      ? () => showShadowDialog(
        context,
        Icons.folder_rounded,
        lang.files,
        Column(children: [
            Text(message.content),
            SizedBox(height: 15.0),
            Container(
              height: 100.0,
              child: fileImage,
            ),
            SizedBox(height: 15.0),
            InkWell(
              onTap: () => OpenFile.open(filePath),
              hoverColor: Colors.transparent,
              child: Container(
                width: 200.0,
                padding: const EdgeInsets.symmetric(vertical: 10.0),
                decoration: BoxDecoration(
                  border: Border.all(color: color.primary),
                  borderRadius: BorderRadius.circular(10.0)),
                child: Center(child: Text(lang.open,
                    style: TextStyle(fontSize: 14.0, color: color.primary))),
              )
            ),
      ]))
      : () => {},
      child: Container(
        padding: const EdgeInsets.symmetric(vertical: 10.0, horizontal: 15.0),
        decoration: BoxDecoration(
          color: const Color(0x40ADB0BB),
          borderRadius: BorderRadius.circular(15.0),
        ),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
            Container(
              height: 36.0,
              child: fileImage,
            ),
            Container(
              padding: const EdgeInsets.only(left: 5.0),
              width: 120.0,
              child: Text(message.content,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: fileExsit
                ? TextStyle(
                  color: color.onPrimary,
                  fontSize: 14.0,
                )
                : TextStyle(
                  color: color.onPrimary.withOpacity(0.5),
                  decoration: TextDecoration.lineThrough,
                  fontSize: 14.0,
              )),
            ),
    ])));
  }

  Widget _showRecord() {
    final raws = message.showRecordTime();
    // text
    return Container(
      width: 120.0,
      padding: const EdgeInsets.symmetric(vertical: 10.0, horizontal: 10.0),
      decoration: BoxDecoration(
        color: const Color(0x40ADB0BB),
        borderRadius: BorderRadius.circular(15.0),
      ),
      child: RecordPlayer(path: Global.recordPath + raws[1], time: raws[0]),
    );
  }

  Widget _showContact(context, lang, color, maxWidth) {
    // contact [name, gid, addr, avatar]
    final infos = message.showContact();
    if (infos[1].length > 0) {
      final gid = 'EH' + infos[1].toUpperCase();
      return GestureDetector(
        onTap: () => showShadowDialog(
          context,
          Icons.person_rounded,
          lang.contactCard,
          UserInfo(
            showQr: false, id: gid, addr: '0x' + infos[2], name: infos[0],
            remark: lang.fromContactCard(name),
            avatar: Avatar(width: 100.0, name: infos[0], avatarPath: infos[3]),
            callback: () {
              Navigator.pop(context);
              Provider.of<ChatProvider>(context, listen: false).requestCreate(
                Request(infos[1], infos[2], infos[0], lang.fromContactCard(name))
              );
            },
          ),
        ),
        child: _showContactCard(
          Avatar(width: 40.0, name: infos[0], avatarPath: infos[3]),
          gid, infos[0], lang.contactCard, color
        )
      );
    } else {
      return _showText(context, color, maxWidth);
    }
  }

  Widget _showInvite(context, lang, color, maxWidth) {
    // contact [type, gid, addr, name, proof, key]
    final infos = message.showInvite();
    if (infos[1].length > 0) {
      final gid = 'EG' + infos[1].toUpperCase();
      final GroupType gtype = infos[0];
      return GestureDetector(
        onTap: () => showShadowDialog(
          context,
          Icons.groups_rounded,
          lang.groupChat,
          UserInfo(showQr: false, id: gid, addr: '0x' + infos[2], name: infos[3],
            title: gtype.lang(lang),
            avatar: Container(width: 100.0, height: 100.0,
              padding: const EdgeInsets.all(8.0),
              decoration: BoxDecoration(color: color.surface, borderRadius: BorderRadius.circular(15.0)),
              child: Icon(Icons.groups_rounded, color: color.primary, size: 36.0),
            ),
            callback: () {
              Navigator.pop(context);
              Provider.of<GroupChatProvider>(context, listen: false).join(
                gtype, infos[1], infos[2], infos[3], fgid, infos[4], infos[5]
              );
            },
          ),
        ),
        child: _showContactCard(
          Container(width: 40.0, height: 40.0,
            decoration: BoxDecoration(color: color.surface, borderRadius: BorderRadius.circular(10.0)),
            child: Icon(Icons.groups_rounded, color: color.primary, size: 20.0),
          ),
          gid, infos[3], lang.groupChat, color)
      );
    } else {
      return _showText(context, color, maxWidth);
    }
  }


  Widget _show(context, color, lang, isDesktop, maxWidth) {
    //final width = MediaQuery.of(context).size.width * 0.6;

    if (message.type == MessageType.String) {
      return _showText(context, color, maxWidth);
    } else if (message.type == MessageType.Image) {
      return _showImage(context, lang, color);
    } else if (message.type == MessageType.File) {
      return _showFile(context, lang, color);
    } else if (message.type == MessageType.Contact) {
      return _showContact(context, lang, color, maxWidth);
    } else if (message.type == MessageType.Record) {
      return  _showRecord();
    } else if (message.type == MessageType.Invite) {
      return  _showInvite(context, lang, color, maxWidth);
    }
    return _showText(context, color, maxWidth);
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);
    final width = MediaQuery.of(context).size.width * 0.6;
    final maxWidth = isDesktop ? width - 300.0 : width;
    final messageShow = _show(context, color, lang, isDesktop, maxWidth);
    final isAvatar = avatar != null && !message.isMe;

    final timeWidget = Container(
      padding: EdgeInsets.only(top: 4.0),
      child: Row(children: [
          if (message.isMe) Spacer(),
          if (isAvatar)
          Container(
            width: 50.0,
            child: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis,
              style: TextStyle(color: color.onPrimary.withOpacity(0.5), fontSize: 10.0)
          )),
          const SizedBox(width: 4.0),
          Text(message.time.toString(), style: TextStyle(
              color: color.onPrimary.withOpacity(0.5),
              fontSize: 10.0)),
          const SizedBox(width: 4.0),
          Icon(
            message.isDelivery == null ? Icons.hourglass_top
            : (message.isDelivery! ? Icons.done : Icons.error),
            size: 10.0,
            color: message.isDelivery == null ? color.primaryVariant
            : (message.isDelivery! ? color.primary : Colors.red)
          ),
    ]));

    final mainWidget = Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Expanded(
          child: Align(
            alignment: message.isMe ? Alignment.topRight : Alignment.topLeft,
            child: messageShow,
          ),
        ),
    ]);

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4.0),
      child:
      isAvatar
      ? Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          avatar!,
          const SizedBox(width: 4.0),
          Expanded(
            child: Column(
              children: [
                mainWidget,
                timeWidget,
              ]
            )
          )
        ]
      )
      : Column(
        children: [
          mainWidget,
          timeWidget,
        ]
      )
    );
  }
}
