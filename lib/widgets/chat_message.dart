import 'dart:io';

import 'package:flutter/material.dart';
import 'package:image_gallery_saver/image_gallery_saver.dart';
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
import 'package:esse/global.dart';

import 'package:esse/apps/primitives.dart';
import 'package:esse/apps/chat/models.dart' show Request;
import 'package:esse/apps/chat/provider.dart';

class ChatMessage extends StatelessWidget {
  final Widget avatar;
  final String name;
  final BaseMessage message;

  const ChatMessage({Key key, this.name, this.message, this.avatar}): super(key: key);

  Widget _showText(context, color, isDesktop) {
    final width = MediaQuery.of(context).size.width * 0.6;
    // text
    return Container(
      constraints: BoxConstraints(minWidth: 50, maxWidth: isDesktop ? width - 300.0 : width),
      padding: const EdgeInsets.symmetric(vertical: 10.0, horizontal: 14.0),
      decoration: BoxDecoration(
        color: message.isMe ? Color(0xFF6174FF) : color.primaryVariant,
        borderRadius: BorderRadius.circular(15.0),
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
                  final result = await ImageGallerySaver.saveFile(imagePath);
                  print(result);
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

  Widget _showContact(context, lang, color) {
    // contact [name, gid, addr, avatar]
    final infos = message.showContact();
    final gid = 'EH' + infos[1].toUpperCase();

    if (infos != null) {
      return GestureDetector(
        onTap: () => showShadowDialog(
          context,
          Icons.person_rounded,
          lang.contact,
          Column(children: [
              Avatar(width: 100.0, name: infos[0], avatarPath: infos[3]),
              const SizedBox(height: 10.0),
              Text(infos[0]),
              const SizedBox(height: 10.0),
              const Divider(height: 1.0, color: Color(0x40ADB0BB)),
              const SizedBox(height: 10.0),
              _infoListTooltip(Icons.person, color.primary, gid),
              _infoListTooltip(Icons.location_on, color.primary, "0x" + infos[2]),
              Container(
                width: 300.0,
                padding: const EdgeInsets.symmetric(vertical: 10.0),
                child: Row(
                  children: [
                    Icon(Icons.turned_in, size: 20.0, color: color.primary),
                    const SizedBox(width: 20.0),
                    Expanded(child: Text(lang.fromContactCard(name))),
                  ]
                ),
              ),
              const SizedBox(height: 20.0),
              InkWell(
                onTap: () {
                  Navigator.pop(context);
                  Provider.of<ChatProvider>(context, listen: false).requestCreate(
                    Request(infos[1], infos[2], infos[0], lang.fromContactCard(name))
                  );
                },
                hoverColor: Colors.transparent,
                child: Container(
                  width: 200.0,
                  padding: const EdgeInsets.symmetric(vertical: 10.0),
                  decoration: BoxDecoration(
                    border: Border.all(color: color.primary),
                    borderRadius: BorderRadius.circular(10.0)),
                  child: Center(child: Text(lang.addFriend,
                      style: TextStyle(fontSize: 14.0, color: color.primary))),
                )
              ),
            ]
          )
        ),
        child: Container(
          padding: const EdgeInsets.symmetric(
            vertical: 10.0, horizontal: 10.0),
          width: 200.0,
          decoration: BoxDecoration(
            color: const Color(0x40ADB0BB),
            borderRadius: BorderRadius.circular(15.0),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(children: [
                  Avatar(width: 40.0, name: infos[0], avatarPath: infos[3]),
                  Container(
                    width: 135.0,
                    padding: const EdgeInsets.only(left: 10.0),
                    child: Column(children: [
                        Text(infos[0],
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(
                            color: color.onPrimary, fontSize: 16.0)),
                        SizedBox(height: 5.0),
                        Text(betterPrint(gid),
                          style: TextStyle(
                            color: Colors.grey, fontSize: 12.0)),
                  ])),
              ]),
              SizedBox(height: 5.0),
              const Divider(height: 1.0, color: Color(0x40ADB0BB)),
              SizedBox(height: 3.0),
              Text(lang.contactCard,
                style: TextStyle(color: Colors.grey, fontSize: 10.0)),
      ])));
    } else {
      return Container(
        padding:
        const EdgeInsets.symmetric(vertical: 10.0, horizontal: 10.0),
        width: 200.0,
        decoration: BoxDecoration(
          color: const Color(0x40ADB0BB),
          borderRadius: BorderRadius.circular(15.0),
        ),
        child:
        Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Row(children: [
                Container(
                  height: 35.0,
                  child: Image(
                    image: AssetImage('assets/images/image_missing.png'),
                    fit: BoxFit.cover),
                ),
                Container(
                  width: 130.0,
                  padding: const EdgeInsets.only(left: 10.0),
                  child: Text(message.content,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      color: color.onPrimary.withOpacity(0.5),
                      decoration: TextDecoration.lineThrough,
                      fontSize: 16.0)),
                ),
            ]),
            SizedBox(height: 5.0),
            const Divider(height: 1.0, color: Color(0x40ADB0BB)),
            SizedBox(height: 3.0),
            Text(lang.contactCard,
              style: TextStyle(color: Colors.grey, fontSize: 10.0)),
      ]));
    }
  }

  Widget _showInvite(context, lang, color) {
    // contact [name, gid, addr, avatar]
    //final infos = message.showContact();
    //final gid = 'EG' + infos[1].toUpperCase();

    final width = MediaQuery.of(context).size.width * 0.6;
    // text
    return Container(
      constraints: BoxConstraints(minWidth: 50, maxWidth: width),
      padding: const EdgeInsets.symmetric(vertical: 10.0, horizontal: 14.0),
      decoration: BoxDecoration(
        color: message.isMe ? Color(0xFF6174FF) : color.primaryVariant,
        borderRadius: BorderRadius.circular(15.0),
      ),
      child: Text(message.content,
        style: TextStyle(
          color: message.isMe ? Colors.white : Color(0xFF1C1939),
          fontSize: 14.0)));
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

  Widget _show(context, color, lang, isDesktop) {
    if (message.type == MessageType.String) {
      return _showText(context, color, isDesktop);
    } else if (message.type == MessageType.Image) {
      return _showImage(context, lang, color);
    } else if (message.type == MessageType.File) {
      return _showFile(context, lang, color);
    } else if (message.type == MessageType.Contact) {
      return _showContact(context, lang, color);
    } else if (message.type == MessageType.Record) {
      return  _showRecord();
    } else if (message.type == MessageType.Invite) {
      return  _showInvite(context, lang, color);
    }
    return _showText(context, color, isDesktop);
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);
    final messageShow = _show(context, color, lang, isDesktop);
    final isAvatar = avatar != null && !message.isMe;

    final timeWidget = Container(
      padding: EdgeInsets.only(top: 6.0),
      child: Row(children: [
          if (message.isMe) Spacer(),
          if (isAvatar)
          Container(
            width: 50.0,
            child: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis,
              style: TextStyle(color: color.onPrimary.withOpacity(0.5), fontSize: 12.0)
          )),
          const SizedBox(width: 4.0),
          Text(message.time.toString(), style: TextStyle(
              color: color.onPrimary.withOpacity(0.5),
              fontSize: 12.0)),
          const SizedBox(width: 4.0),
          Icon(
            message.isDelivery == null ? Icons.hourglass_top
            : (message.isDelivery ? Icons.done : Icons.error),
            size: 12.0,
            color: message.isDelivery == null ? color.primaryVariant
            : (message.isDelivery ? color.primary : Colors.red)
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
      padding: const EdgeInsets.symmetric(vertical: 5.0),
      child:
      isAvatar
      ? Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          avatar,
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
